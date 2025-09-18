use axum::{extract::State, response::Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use uuid::Uuid;
use validator::Validate;
use tracing::{info, warn};

use crate::{
    models::*,
    handlers::AppState,
    auth::{Claims, BEARER},
    errors::{ApiError, ApiResult},
};

pub async fn register_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Validate request
    request.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;
    
    // Check if username already exists
    let existing_user = sqlx::query_scalar!(
        "SELECT id FROM users WHERE username = ?1",
        request.username
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    if existing_user.is_some() {
        return Err(ApiError::Conflict("Username already exists".to_string()));
    }
    
    // Check if Near account already exists (if provided)
    if let Some(ref near_account) = request.near_account_id {
        let existing_near = sqlx::query_scalar!(
            "SELECT id FROM users WHERE near_account_id = ?1",
            near_account
        )
        .fetch_optional(&state.db_pool)
        .await
        .map_err(|e| ApiError::Database(e.to_string()))?;
        
        if existing_near.is_some() {
            return Err(ApiError::Conflict("Near account already registered".to_string()));
        }
    }
    
    // Hash password
    let password_hash = hash(&request.password, DEFAULT_COST)
        .map_err(|e| ApiError::Internal(format!("Password hashing failed: {}", e)))?;
    
    // Create user
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (
            id, username, email, password_hash, near_account_id,
            is_active, is_admin, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        RETURNING *
        "#,
        user_id,
        request.username,
        request.email,
        password_hash,
        request.near_account_id,
        true,
        false,
        now,
        now
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    info!("New user registered: {} ({})", user.username, user.id);
    
    // Generate JWT token
    let token = generate_jwt_token(&state, &user)?;
    
    let response = AuthResponse {
        access_token: token,
        token_type: BEARER.to_string(),
        expires_in: 3600, // 1 hour
        user: UserProfile {
            id: user.id,
            username: user.username,
            email: user.email,
            near_account_id: user.near_account_id,
            is_admin: user.is_admin,
            created_at: user.created_at,
        },
    };
    
    Ok(Json(response))
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Find user by username
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE username = ?1 AND is_active = true",
        request.username
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::Unauthorized("Invalid credentials".to_string()))?;
    
    // Verify password
    let password_hash = user.password_hash
        .ok_or_else(|| ApiError::BadRequest("Password login not available for this account".to_string()))?;
    
    let password_valid = verify(&request.password, &password_hash)
        .map_err(|e| ApiError::Internal(format!("Password verification failed: {}", e)))?;
    
    if !password_valid {
        warn!("Failed login attempt for user: {}", request.username);
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }
    
    // Update last login
    sqlx::query!(
        "UPDATE users SET last_login_at = ?1 WHERE id = ?2",
        Utc::now(),
        user.id
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    info!("User logged in: {} ({})", user.username, user.id);
    
    // Generate JWT token
    let token = generate_jwt_token(&state, &user)?;
    
    let response = AuthResponse {
        access_token: token,
        token_type: BEARER.to_string(),
        expires_in: 3600, // 1 hour
        user: UserProfile {
            id: user.id,
            username: user.username,
            email: user.email,
            near_account_id: user.near_account_id,
            is_admin: user.is_admin,
            created_at: user.created_at,
        },
    };
    
    Ok(Json(response))
}

pub async fn near_wallet_login(
    State(state): State<AppState>,
    Json(request): Json<NearWalletLoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Verify Near wallet signature
    if !verify_near_signature(&request).await? {
        return Err(ApiError::Unauthorized("Invalid Near wallet signature".to_string()));
    }
    
    // Find or create user by Near account ID
    let user = match sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE near_account_id = ?1 AND is_active = true",
        request.account_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    {
        Some(user) => {
            // Update last login
            sqlx::query!(
                "UPDATE users SET last_login_at = ?1 WHERE id = ?2",
                Utc::now(),
                user.id
            )
            .execute(&state.db_pool)
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?;
            
            user
        }
        None => {
            // Create new user
            let user_id = Uuid::new_v4();
            let now = Utc::now();
            let username = format!("near_{}", request.account_id.replace('.', "_"));
            
            sqlx::query_as!(
                User,
                r#"
                INSERT INTO users (
                    id, username, near_account_id, is_active, is_admin,
                    created_at, updated_at, last_login_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                RETURNING *
                "#,
                user_id,
                username,
                request.account_id,
                true,
                false,
                now,
                now,
                now
            )
            .fetch_one(&state.db_pool)
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?
        }
    };
    
    info!("Near wallet login: {} ({})", user.username, user.id);
    
    // Generate JWT token
    let token = generate_jwt_token(&state, &user)?;
    
    let response = AuthResponse {
        access_token: token,
        token_type: BEARER.to_string(),
        expires_in: 3600, // 1 hour
        user: UserProfile {
            id: user.id,
            username: user.username,
            email: user.email,
            near_account_id: user.near_account_id,
            is_admin: user.is_admin,
            created_at: user.created_at,
        },
    };
    
    Ok(Json(response))
}

// Helper functions

fn generate_jwt_token(state: &AppState, user: &User) -> ApiResult<String> {
    let expiration = Utc::now() + Duration::hours(1);
    
    let claims = Claims {
        user_id: user.id,
        username: user.username.clone(),
        near_account_id: user.near_account_id.clone(),
        is_admin: user.is_admin,
        exp: expiration.timestamp() as usize,
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_ref()),
    )
    .map_err(|e| ApiError::Internal(format!("Token generation failed: {}", e)))?;
    
    Ok(token)
}

async fn verify_near_signature(request: &NearWalletLoginRequest) -> ApiResult<bool> {
    use near_crypto::{PublicKey, Signature};
    use std::str::FromStr;
    
    // Parse public key and signature
    let public_key = PublicKey::from_str(&request.public_key)
        .map_err(|e| ApiError::BadRequest(format!("Invalid public key: {}", e)))?;
    
    let signature = Signature::from_str(&request.signature)
        .map_err(|e| ApiError::BadRequest(format!("Invalid signature: {}", e)))?;
    
    // Verify signature
    let message_bytes = request.message.as_bytes();
    let is_valid = signature.verify(message_bytes, &public_key);
    
    Ok(is_valid)
}