use axum::{extract::State, http::StatusCode, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use near_crypto::{PublicKey, Signature};
use near_primitives::account::id::AccountId;
use uuid::Uuid;
use crate::{
    database::{create_user, get_user_by_username, get_user_by_account_id, create_api_key, verify_api_key},
    errors::{ApiError, ApiResult},
    models::{User, CreateUserRequest, LoginRequest, NearLoginRequest, AuthResponse, ApiKey},
    handlers::AppState,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // User ID
    pub username: String,      // Username
    pub account_id: Option<String>, // Near account ID
    pub exp: usize,            // Expiration time
    pub iat: usize,            // Issued at time
    pub token_type: String,    // "access" or "api_key"
}

impl Claims {
    pub fn new(user: &User, token_type: &str, duration_hours: i64) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::hours(duration_hours)).timestamp() as usize;
        
        Self {
            sub: user.id.to_string(),
            username: user.username.clone(),
            account_id: user.near_account_id.clone(),
            exp,
            iat: now.timestamp() as usize,
            token_type: token_type.to_string(),
        }
    }
}

pub fn create_jwt_token(user: &User, secret: &str, token_type: &str) -> ApiResult<String> {
    let duration = if token_type == "api_key" { 24 * 30 } else { 24 }; // 30 days for API keys, 24 hours for access tokens
    let claims = Claims::new(user, token_type, duration);
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| ApiError::Internal(format!("Failed to create JWT: {}", e)))
}

pub fn verify_jwt_token(token: &str, secret: &str) -> ApiResult<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| ApiError::Unauthorized(format!("Invalid token: {}", e)))
}

pub async fn register_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Validate username
    if request.username.len() < 3 || request.username.len() > 50 {
        return Err(ApiError::BadRequest("Username must be 3-50 characters".to_string()));
    }
    
    // Validate password
    if request.password.len() < 8 {
        return Err(ApiError::BadRequest("Password must be at least 8 characters".to_string()));
    }

    // Check if username already exists
    if get_user_by_username(&state.db_pool, &request.username).await.is_ok() {
        return Err(ApiError::Conflict("Username already exists".to_string()));
    }

    // Validate Near account ID if provided
    if let Some(ref account_id) = request.near_account_id {
        if account_id.parse::<AccountId>().is_err() {
            return Err(ApiError::BadRequest("Invalid Near account ID format".to_string()));
        }
        
        // Check if Near account is already registered
        if get_user_by_account_id(&state.db_pool, account_id).await.is_ok() {
            return Err(ApiError::Conflict("Near account already registered".to_string()));
        }
    }

    // Hash password
    let password_hash = hash(&request.password, DEFAULT_COST)
        .map_err(|e| ApiError::Internal(format!("Failed to hash password: {}", e)))?;

    // Create user in database
    let user = create_user(
        &state.db_pool,
        &request.username,
        &request.email,
        &password_hash,
        request.near_account_id.as_deref(),
    ).await?;

    // Generate JWT token
    let access_token = create_jwt_token(&user, &state.config.jwt_secret, "access")?;

    Ok(Json(AuthResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 86400, // 24 hours
        user: user.into(),
    }))
}

pub async fn login_user(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Get user from database
    let user = get_user_by_username(&state.db_pool, &request.username).await
        .map_err(|_| ApiError::Unauthorized("Invalid credentials".to_string()))?;

    // Verify password
    if !verify(&request.password, &user.password_hash)
        .map_err(|e| ApiError::Internal(format!("Failed to verify password: {}", e)))? {
        return Err(ApiError::Unauthorized("Invalid credentials".to_string()));
    }

    // Generate JWT token
    let access_token = create_jwt_token(&user, &state.config.jwt_secret, "access")?;

    Ok(Json(AuthResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 86400, // 24 hours
        user: user.into(),
    }))
}

pub async fn near_wallet_login(
    State(state): State<AppState>,
    Json(request): Json<NearLoginRequest>,
) -> ApiResult<Json<AuthResponse>> {
    // Parse Near account ID
    let account_id: AccountId = request.account_id.parse()
        .map_err(|_| ApiError::BadRequest("Invalid Near account ID".to_string()))?;

    // Parse public key
    let public_key: PublicKey = request.public_key.parse()
        .map_err(|_| ApiError::BadRequest("Invalid public key format".to_string()))?;

    // Parse signature
    let signature: Signature = request.signature.parse()
        .map_err(|_| ApiError::BadRequest("Invalid signature format".to_string()))?;

    // Verify signature
    let message_bytes = request.message.as_bytes();
    if !signature.verify(message_bytes, &public_key) {
        return Err(ApiError::Unauthorized("Invalid signature".to_string()));
    }

    // Check if message is recent (within 5 minutes)
    let message_parts: Vec<&str> = request.message.split('|').collect();
    if message_parts.len() != 2 {
        return Err(ApiError::BadRequest("Invalid message format".to_string()));
    }
    
    let timestamp: i64 = message_parts[1].parse()
        .map_err(|_| ApiError::BadRequest("Invalid timestamp in message".to_string()))?;
    
    let now = Utc::now().timestamp();
    if (now - timestamp).abs() > 300 { // 5 minutes
        return Err(ApiError::Unauthorized("Message timestamp too old".to_string()));
    }

    // Get or create user
    let user = match get_user_by_account_id(&state.db_pool, &request.account_id).await {
        Ok(user) => user,
        Err(_) => {
            // Create new user with Near account
            let username = format!("near_{}", account_id.as_str().replace('.', "_"));
            let email = format!("{}@near.local", account_id.as_str());
            let dummy_password = hash("", DEFAULT_COST)
                .map_err(|e| ApiError::Internal(format!("Failed to hash password: {}", e)))?;
            
            create_user(
                &state.db_pool,
                &username,
                &email,
                &dummy_password,
                Some(&request.account_id),
            ).await?
        }
    };

    // Generate JWT token
    let access_token = create_jwt_token(&user, &state.config.jwt_secret, "access")?;

    Ok(Json(AuthResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: 86400, // 24 hours
        user: user.into(),
    }))
}

pub async fn create_user_api_key(
    user_id: Uuid,
    name: String,
    expires_in_days: Option<i32>,
    state: &AppState,
) -> ApiResult<ApiKey> {
    let expires_at = expires_in_days.map(|days| Utc::now() + Duration::days(days as i64));
    
    // Generate API key token
    let user = crate::database::get_user_by_id(&state.db_pool, user_id).await?;
    let token = create_jwt_token(&user, &state.config.jwt_secret, "api_key")?;
    
    create_api_key(
        &state.db_pool,
        user_id,
        &name,
        &token,
        expires_at,
    ).await
}

pub async fn verify_user_api_key(token: &str, state: &AppState) -> ApiResult<(User, ApiKey)> {
    // Verify JWT token first
    let claims = verify_jwt_token(token, &state.config.jwt_secret)?;
    
    if claims.token_type != "api_key" {
        return Err(ApiError::Unauthorized("Invalid token type".to_string()));
    }
    
    // Verify API key in database
    let api_key = verify_api_key(&state.db_pool, token).await?;
    let user = crate::database::get_user_by_id(&state.db_pool, api_key.user_id).await?;
    
    Ok((user, api_key))
}