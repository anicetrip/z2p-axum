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

#[derive(serde::Deserialize)]
struct FormData {
    email: String,
    name: String,
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
