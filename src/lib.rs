

use axum::{Router, routing::get};
use tokio::net::TcpListener;

// 健康检查处理器
async fn health_check() -> axum::http::StatusCode {
    axum::http::StatusCode::OK
}

pub async fn run(listener: TcpListener) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new().route("/health_check", get(health_check));
    axum::serve(listener, app).await?;
    Ok(())
}