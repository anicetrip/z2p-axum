pub mod entity; // 新增这一行，声明 entity 模块


use axum::{
    Form, Router,
    response::IntoResponse,
    routing::{get, post},
};
use reqwest::StatusCode;
use tokio::net::TcpListener;

// 健康检查处理器
async fn health_check() -> axum::http::StatusCode {
    axum::http::StatusCode::OK
}

#[derive(serde::Deserialize, Debug)] // 添加 Debug 便于日志打印
pub struct FormData {
    pub email: String,
    pub name: String,
}

async fn subscribe(Form(form): Form<FormData>) -> impl IntoResponse {
    println!("Name: {}, Email: {}", form.name, form.email);
    (StatusCode::OK, "Subscribed!")
}

pub async fn run(listener: TcpListener) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe));
    axum::serve(listener, app).await?;
    Ok(())
}
