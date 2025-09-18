use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// User models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub near_account_id: Option<String>,
    pub email: Option<String>,
    pub username: String,
    pub password_hash: Option<String>,
    pub is_active: bool,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 8))]
    pub password: String,
    pub near_account_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NearWalletLoginRequest {
    pub account_id: String,
    pub public_key: String,
    pub signature: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserProfile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub near_account_id: Option<String>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

// API Key models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub prefix: String,
    pub is_active: bool,
    pub rate_limit_override: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateApiKeyRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub rate_limit_override: Option<i32>,
    pub expires_in_days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub key: Option<String>, // Only returned on creation
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

// Task models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: Uuid,
    pub user_id: Uuid,
    pub contract_task_id: Option<i64>,
    pub task_type: String,
    pub model_name: String,
    pub input_data: String,
    pub parameters: Option<String>, // JSON
    pub status: TaskStatus,
    pub priority: i32,
    pub estimated_cost: String,
    pub actual_cost: Option<String>,
    pub assigned_node_id: Option<String>,
    pub result_data: Option<String>,
    pub proof_hash: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "task_status", rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Submitted,
    Assigned,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Expired,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SubmitTaskRequest {
    #[validate(length(min = 1, max = 50))]
    pub task_type: String,
    #[validate(length(min = 1, max = 200))]
    pub model_name: String,
    #[validate(length(min = 1, max = 50000))]
    pub input_data: String,
    pub parameters: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub max_cost: Option<String>, // In yoctoNEAR
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub task_type: String,
    pub model_name: String,
    pub status: TaskStatus,
    pub priority: i32,
    pub estimated_cost: String,
    pub actual_cost: Option<String>,
    pub assigned_node_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResultResponse {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub result_data: Option<serde_json::Value>,
    pub proof_hash: Option<String>,
    pub error_message: Option<String>,
    pub execution_time_ms: Option<i64>,
    pub completed_at: Option<DateTime<Utc>>,
}

// Node models
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    pub account_id: String,
    pub public_ip: String,
    pub gpu_specs: String,
    pub cpu_specs: String,
    pub api_endpoint: String,
    pub is_active: bool,
    pub last_heartbeat: DateTime<Utc>,
    pub total_tasks_completed: u64,
    pub reputation_score: u32,
    pub stake_amount: String,
}

// Usage and statistics models
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserUsageStats {
    pub user_id: Uuid,
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub failed_tasks: i64,
    pub total_cost: String,
    pub current_month_tasks: i64,
    pub current_month_cost: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_nodes: u32,
    pub active_nodes: u32,
    pub total_tasks: u64,
    pub pending_tasks: u32,
    pub completed_tasks_24h: u32,
    pub average_task_time_ms: Option<f64>,
    pub total_staked_near: String,
    pub network_utilization_percent: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub active_users: u32,
    pub api_requests_24h: u64,
    pub error_rate_percent: f32,
    pub average_response_time_ms: f64,
    pub database_connections: u32,
    pub redis_connections: u32,
    pub uptime_seconds: u64,
}

// Error response models
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub code: Option<String>,
    pub details: Option<serde_json::Value>,
}

// Pagination models
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub limit: u32,
    pub total: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationQuery {
    pub fn normalize(&self) -> (u32, u32) {
        let page = self.page.unwrap_or(1).max(1);
        let limit = self.limit.unwrap_or(20).clamp(1, 100);
        (page, limit)
    }
    
    pub fn offset(&self) -> u32 {
        let (page, limit) = self.normalize();
        (page - 1) * limit
    }
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: u32, limit: u32, total: u64) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
        
        Self {
            data,
            pagination: PaginationInfo {
                page,
                limit,
                total,
                total_pages,
                has_next: page < total_pages,
                has_prev: page > 1,
            },
        }
    }
}