use axum::{
    Router,
    routing::{get, post},
};
use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;

use crate::routes::{health_check, subscribe};

pub async fn run(
    listener: TcpListener,
    connection: DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(connection); // 添加 SeaORM 数据库连接到应用状态;
    axum::serve(listener, app).await?;
    Ok(())
}
