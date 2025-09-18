use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::net::IpAddr;
use crate::{
    auth::{verify_jwt_token, verify_user_api_key},
    rate_limit::RateLimiter,
    errors::{ApiError, ApiResult},
    handlers::AppState,
    models::User,
};

// Extension types for storing user info in request
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user: User,
    pub is_api_key: bool,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip authentication for public routes
    let path = request.uri().path();
    if is_public_route(path) {
        return Ok(next.run(request).await);
    }

    // Extract authorization header
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Parse Bearer token
    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Try to authenticate with JWT token first, then API key
    let authenticated_user = match verify_jwt_token(token, &state.config.jwt_secret) {
        Ok(claims) => {
            if claims.token_type == "api_key" {
                // Verify API key in database
                match verify_user_api_key(token, &state).await {
                    Ok((user, _api_key)) => AuthenticatedUser {
                        user,
                        is_api_key: true,
                    },
                    Err(_) => return Err(StatusCode::UNAUTHORIZED),
                }
            } else {
                // Regular JWT access token
                let user = match crate::database::get_user_by_id(
                    &state.db_pool,
                    uuid::Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?,
                ).await {
                    Ok(user) => user,
                    Err(_) => return Err(StatusCode::UNAUTHORIZED),
                };
                
                AuthenticatedUser {
                    user,
                    is_api_key: false,
                }
            }
        }
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Check if user is active
    if !authenticated_user.user.is_active {
        return Err(StatusCode::FORBIDDEN);
    }

    // Store user info in request extensions
    request.extensions_mut().insert(authenticated_user);

    Ok(next.run(request).await)
}

pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let rate_limiter = RateLimiter::new(state.redis_client.clone());
    
    // Get user info if authenticated
    let user_info = request.extensions().get::<AuthenticatedUser>().cloned();
    
    // Determine rate limit identifier and config
    let (identifier, config) = if let Some(auth_user) = &user_info {
        // Use user-based rate limiting
        let tier = &auth_user.user.tier;
        let config = rate_limiter.get_user_tier_config(tier);
        (format!("user:{}", auth_user.user.id), config)
    } else {
        // Use IP-based rate limiting for unauthenticated requests
        let ip = get_client_ip(&request);
        let config = rate_limiter.get_ip_rate_limit_config();
        (format!("ip:{}", ip), config)
    };
    
    // Check rate limit
    match rate_limiter.check_rate_limit(&identifier, &config).await {
        Ok(info) => {
            let mut response = next.run(request).await;
            
            // Add rate limit headers
            let headers = response.headers_mut();
            headers.insert("X-RateLimit-Limit", info.limit.to_string().parse().unwrap());
            headers.insert("X-RateLimit-Remaining", info.remaining.to_string().parse().unwrap());
            headers.insert("X-RateLimit-Reset", info.reset_time.to_string().parse().unwrap());
            
            if let Some(retry_after) = info.retry_after {
                if info.remaining == 0 {
                    headers.insert("Retry-After", retry_after.to_string().parse().unwrap());
                    return Err(StatusCode::TOO_MANY_REQUESTS);
                }
            }
            
            Ok(response)
        }
        Err(_) => {
            // Rate limit exceeded
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
    }
}

pub async fn admin_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check if user is authenticated and is admin
    let auth_user = request
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_user.user.is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(request).await)
}

fn is_public_route(path: &str) -> bool {
    matches!(
        path,
        "/health"
            | "/api/v1/auth/register"
            | "/api/v1/auth/login"
            | "/api/v1/auth/near-login"
            | "/api/v1/network/stats"
            | "/api/v1/nodes"
    ) || path.starts_with("/api/v1/nodes/") && !path.contains("/admin/")
}

fn get_client_ip(request: &Request) -> IpAddr {
    // Try to get real IP from headers (for proxy setups)
    if let Some(forwarded_for) = request.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }
    
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return ip;
            }
        }
    }
    
    // Fallback to connection info (this might not be available in some setups)
    // For now, return localhost as fallback
    "127.0.0.1".parse().unwrap()
}

// Utility function to extract authenticated user from request
pub fn get_authenticated_user(request: &Request) -> ApiResult<&AuthenticatedUser> {
    request
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or_else(|| ApiError::Unauthorized("User not authenticated".to_string()))
}

// Utility function to check if user has admin privileges
pub fn require_admin(auth_user: &AuthenticatedUser) -> ApiResult<()> {
    if !auth_user.user.is_admin {
        return Err(ApiError::Forbidden("Admin privileges required".to_string()));
    }
    Ok(())
}

// Utility function to check if user can access resource
pub fn check_user_access(auth_user: &AuthenticatedUser, resource_user_id: uuid::Uuid) -> ApiResult<()> {
    if auth_user.user.id != resource_user_id && !auth_user.user.is_admin {
        return Err(ApiError::Forbidden("Access denied".to_string()));
    }
    Ok(())
}