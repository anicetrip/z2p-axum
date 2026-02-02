use axum::{
    Router,
    routing::{get, post},
};
use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info_span;
use uuid::Uuid;

use crate::routes::{health_check, subscribe};

pub async fn run(
    listener: TcpListener,
    connection: DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(connection)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                let request_id = Uuid::new_v4().to_string();
                info_span!(
                    "http_request",
                    request_id = %request_id,
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version()
                )
            }),
        );

    axum::serve(listener, app).await?;
    Ok(())
}
