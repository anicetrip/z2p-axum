// src/routes/health_check.rs
use axum::http::StatusCode;
use tracing::info;

#[tracing::instrument(name = "Processing health check request", skip())]
pub async fn health_check() -> StatusCode {
    // 直接记录日志，request_id 会自动从父 span 继承
    info!("Health check completed successfully");

    StatusCode::OK
}
