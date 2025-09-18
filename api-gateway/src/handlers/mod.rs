use sqlx::SqlitePool;
use redis::Client as RedisClient;
use std::sync::Arc;
use crate::{config::AppConfig, near_client::NearClient};

pub mod auth;
pub mod tasks;
pub mod nodes;
pub mod network;
pub mod users;
pub mod admin;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db_pool: SqlitePool,
    pub redis_client: RedisClient,
    pub near_client: Arc<NearClient>,
}