use axum::{Router, routing::get};
use tokio::net::TcpListener;


// 健康检查处理器
async fn health_check() -> axum::http::StatusCode {
    axum::http::StatusCode::OK
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8000").await?;
    let app = Router::new().route("/health_check", get(health_check));
    axum::serve(listener, app).await?;
    Ok(())
}