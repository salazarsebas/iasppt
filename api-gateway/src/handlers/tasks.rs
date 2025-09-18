use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;
use chrono::Utc;
use validator::Validate;
use tracing::{info, warn, error};

use crate::{
    models::*,
    handlers::AppState,
    auth::Claims,
    errors::{ApiError, ApiResult},
};

pub async fn submit_task(
    State(state): State<AppState>,
    claims: Claims,
    Json(request): Json<SubmitTaskRequest>,
) -> ApiResult<Json<TaskResponse>> {
    // Validate request
    request.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;
    
    info!("Submitting task for user {}: {} with model {}", 
          claims.user_id, request.task_type, request.model_name);
    
    // Check user's current task quota
    let active_tasks = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tasks WHERE user_id = ?1 AND status IN ('pending', 'submitted', 'assigned', 'in_progress')",
        claims.user_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    if active_tasks >= state.config.rate_limits.max_concurrent_tasks as i64 {
        return Err(ApiError::TooManyRequests(
            "Maximum concurrent tasks exceeded".to_string()
        ));
    }
    
    // Estimate cost (simplified - could be more sophisticated)
    let estimated_cost = estimate_task_cost(&request);
    
    // Create task record
    let task_id = Uuid::new_v4();
    let expires_at = Utc::now() + chrono::Duration::hours(24); // 24-hour expiry
    
    let task = sqlx::query_as!(
        Task,
        r#"
        INSERT INTO tasks (
            id, user_id, task_type, model_name, input_data, parameters,
            status, priority, estimated_cost, expires_at, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7, ?8, ?9, ?10)
        RETURNING *
        "#,
        task_id,
        claims.user_id,
        request.task_type,
        request.model_name,
        request.input_data,
        request.parameters.map(|p| p.to_string()),
        request.priority.unwrap_or(0),
        estimated_cost,
        expires_at,
        Utc::now()
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    // Submit task to Near contract
    match submit_task_to_near(&state, &task).await {
        Ok(contract_task_id) => {
            // Update task with contract task ID
            sqlx::query!(
                "UPDATE tasks SET contract_task_id = ?1, status = 'submitted' WHERE id = ?2",
                contract_task_id,
                task_id
            )
            .execute(&state.db_pool)
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?;
            
            info!("Task {} submitted to Near contract with ID {}", task_id, contract_task_id);
        }
        Err(e) => {
            // Mark task as failed
            sqlx::query!(
                "UPDATE tasks SET status = 'failed', error_message = ?1 WHERE id = ?2",
                e.to_string(),
                task_id
            )
            .execute(&state.db_pool)
            .await
            .map_err(|e| ApiError::Database(e.to_string()))?;
            
            error!("Failed to submit task {} to Near: {}", task_id, e);
            return Err(ApiError::Internal("Failed to submit task to blockchain".to_string()));
        }
    }
    
    let response = TaskResponse {
        id: task.id,
        task_type: task.task_type,
        model_name: task.model_name,
        status: TaskStatus::Submitted,
        priority: task.priority,
        estimated_cost: task.estimated_cost,
        actual_cost: None,
        assigned_node_id: None,
        created_at: task.created_at,
        started_at: None,
        completed_at: None,
        expires_at: task.expires_at,
    };
    
    Ok(Json(response))
}

pub async fn get_task(
    State(state): State<AppState>,
    claims: Claims,
    Path(task_id): Path<Uuid>,
) -> ApiResult<Json<TaskResponse>> {
    let task = sqlx::query_as!(
        Task,
        "SELECT * FROM tasks WHERE id = ?1 AND user_id = ?2",
        task_id,
        claims.user_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Task not found".to_string()))?;
    
    let response = TaskResponse {
        id: task.id,
        task_type: task.task_type,
        model_name: task.model_name,
        status: task.status,
        priority: task.priority,
        estimated_cost: task.estimated_cost,
        actual_cost: task.actual_cost,
        assigned_node_id: task.assigned_node_id,
        created_at: task.created_at,
        started_at: task.started_at,
        completed_at: task.completed_at,
        expires_at: task.expires_at,
    };
    
    Ok(Json(response))
}

pub async fn get_task_result(
    State(state): State<AppState>,
    claims: Claims,
    Path(task_id): Path<Uuid>,
) -> ApiResult<Json<TaskResultResponse>> {
    let task = sqlx::query_as!(
        Task,
        "SELECT * FROM tasks WHERE id = ?1 AND user_id = ?2",
        task_id,
        claims.user_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Task not found".to_string()))?;
    
    let execution_time_ms = if let (Some(started), Some(completed)) = (task.started_at, task.completed_at) {
        Some((completed - started).num_milliseconds())
    } else {
        None
    };
    
    let result_data = if let Some(ref data) = task.result_data {
        serde_json::from_str(data).ok()
    } else {
        None
    };
    
    let response = TaskResultResponse {
        task_id: task.id,
        status: task.status,
        result_data,
        proof_hash: task.proof_hash,
        error_message: task.error_message,
        execution_time_ms,
        completed_at: task.completed_at,
    };
    
    Ok(Json(response))
}

pub async fn list_user_tasks(
    State(state): State<AppState>,
    claims: Claims,
    Query(pagination): Query<PaginationQuery>,
) -> ApiResult<Json<PaginatedResponse<TaskResponse>>> {
    let (page, limit) = pagination.normalize();
    let offset = pagination.offset();
    
    // Get total count
    let total = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM tasks WHERE user_id = ?1",
        claims.user_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))? as u64;
    
    // Get tasks
    let tasks = sqlx::query_as!(
        Task,
        "SELECT * FROM tasks WHERE user_id = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3",
        claims.user_id,
        limit,
        offset
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    let task_responses: Vec<TaskResponse> = tasks.into_iter().map(|task| TaskResponse {
        id: task.id,
        task_type: task.task_type,
        model_name: task.model_name,
        status: task.status,
        priority: task.priority,
        estimated_cost: task.estimated_cost,
        actual_cost: task.actual_cost,
        assigned_node_id: task.assigned_node_id,
        created_at: task.created_at,
        started_at: task.started_at,
        completed_at: task.completed_at,
        expires_at: task.expires_at,
    }).collect();
    
    let response = PaginatedResponse::new(task_responses, page, limit, total);
    Ok(Json(response))
}

pub async fn cancel_task(
    State(state): State<AppState>,
    claims: Claims,
    Path(task_id): Path<Uuid>,
) -> ApiResult<Json<TaskResponse>> {
    let task = sqlx::query_as!(
        Task,
        "SELECT * FROM tasks WHERE id = ?1 AND user_id = ?2",
        task_id,
        claims.user_id
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?
    .ok_or_else(|| ApiError::NotFound("Task not found".to_string()))?;
    
    // Check if task can be cancelled
    match task.status {
        TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
            return Err(ApiError::BadRequest("Task cannot be cancelled".to_string()));
        }
        _ => {}
    }
    
    // Update task status
    let updated_task = sqlx::query_as!(
        Task,
        "UPDATE tasks SET status = 'cancelled' WHERE id = ?1 RETURNING *",
        task_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| ApiError::Database(e.to_string()))?;
    
    info!("Task {} cancelled by user {}", task_id, claims.user_id);
    
    let response = TaskResponse {
        id: updated_task.id,
        task_type: updated_task.task_type,
        model_name: updated_task.model_name,
        status: updated_task.status,
        priority: updated_task.priority,
        estimated_cost: updated_task.estimated_cost,
        actual_cost: updated_task.actual_cost,
        assigned_node_id: updated_task.assigned_node_id,
        created_at: updated_task.created_at,
        started_at: updated_task.started_at,
        completed_at: updated_task.completed_at,
        expires_at: updated_task.expires_at,
    };
    
    Ok(Json(response))
}

// Helper functions

fn estimate_task_cost(request: &SubmitTaskRequest) -> String {
    // Simple cost estimation based on task type and model
    // In production, this would be more sophisticated
    let base_cost = match request.task_type.as_str() {
        "inference" => 10_000_000_000_000_000_000_000u128, // 0.01 NEAR
        "text_generation" => 50_000_000_000_000_000_000_000u128, // 0.05 NEAR
        "classification" => 20_000_000_000_000_000_000_000u128, // 0.02 NEAR
        "embedding" => 15_000_000_000_000_000_000_000u128, // 0.015 NEAR
        _ => 25_000_000_000_000_000_000_000u128, // 0.025 NEAR default
    };
    
    // Adjust based on input size
    let input_multiplier = (request.input_data.len() as f64 / 1000.0).max(1.0);
    let final_cost = (base_cost as f64 * input_multiplier) as u128;
    
    final_cost.to_string()
}

async fn submit_task_to_near(state: &AppState, task: &Task) -> anyhow::Result<i64> {
    // Convert task to Near contract format
    let task_description = serde_json::json!({
        "model": task.model_name,
        "input": task.input_data,
        "task_type": task.task_type,
        "parameters": task.parameters.as_ref().and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
    });
    
    // Submit to Near contract
    let result = state.near_client
        .submit_task(task_description.to_string(), task.estimated_cost.parse()?)
        .await?;
    
    // Extract task ID from transaction result
    // This would need to parse the actual transaction result
    Ok(task.id.as_u128() as i64)
}