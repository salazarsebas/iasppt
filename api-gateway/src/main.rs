use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    limit::RequestBodyLimitLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handlers;
mod auth;
mod models;
mod database;
mod near_client;
mod rate_limit;
mod middleware;
mod errors;

use config::AppConfig;
use handlers::*;
use middleware::{auth_middleware, rate_limit_middleware};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "deai_api_gateway=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::load()?;
    
    // Initialize database
    let db_pool = database::init_database(&config.database_url).await?;
    
    // Initialize Redis for rate limiting
    let redis_client = redis::Client::open(config.redis_url.clone())?;
    
    // Initialize Near client
    let near_client = near_client::NearClient::new(&config).await?;
    
    // Build application state
    let app_state = handlers::AppState {
        config: config.clone(),
        db_pool,
        redis_client,
        near_client: std::sync::Arc::new(near_client),
    };

    // Build our application with routes
    let app = Router::new()
        // Public routes
        .route("/health", get(health_check))
        .route("/api/v1/auth/register", post(auth::register_user))
        .route("/api/v1/auth/login", post(auth::login_user))
        .route("/api/v1/auth/near-login", post(auth::near_wallet_login))
        
        // Protected routes
        .route("/api/v1/tasks", post(tasks::submit_task))
        .route("/api/v1/tasks/:task_id", get(tasks::get_task))
        .route("/api/v1/tasks/:task_id/result", get(tasks::get_task_result))
        .route("/api/v1/tasks", get(tasks::list_user_tasks))
        .route("/api/v1/tasks/:task_id/cancel", post(tasks::cancel_task))
        
        // Node information
        .route("/api/v1/nodes", get(nodes::list_active_nodes))
        .route("/api/v1/nodes/:node_id", get(nodes::get_node_info))
        .route("/api/v1/network/stats", get(network::get_network_stats))
        
        // User account management
        .route("/api/v1/user/profile", get(users::get_profile))
        .route("/api/v1/user/api-keys", post(users::create_api_key))
        .route("/api/v1/user/api-keys", get(users::list_api_keys))
        .route("/api/v1/user/api-keys/:key_id", post(users::revoke_api_key))
        .route("/api/v1/user/usage", get(users::get_usage_stats))
        
        // Admin routes
        .route("/api/v1/admin/users", get(admin::list_users))
        .route("/api/v1/admin/tasks", get(admin::list_all_tasks))
        .route("/api/v1/admin/nodes", get(admin::list_all_nodes))
        .route("/api/v1/admin/system/metrics", get(admin::get_system_metrics))
        
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB limit
                .layer(axum::middleware::from_fn_with_state(
                    app_state.clone(),
                    rate_limit_middleware,
                ))
                .layer(axum::middleware::from_fn_with_state(
                    app_state.clone(),
                    auth_middleware,
                )),
        )
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("🚀 DeAI API Gateway listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}