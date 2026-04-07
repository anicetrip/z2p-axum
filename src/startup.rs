use axum::{
    Router,
    routing::{get, post},
    serve::Serve,
};
use sea_orm::{Database, DatabaseConnection};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info_span;
use uuid::Uuid;

use crate::configuration::DatabaseSettings;
use crate::{
    configuration::Settings,
    routes::{health_check, subscribe},
};

pub fn run(
    listener: TcpListener,
    connection: DatabaseConnection,
) -> Serve<TcpListener, Router, Router> {
    let router = build_router(connection);
    axum::serve(listener, router)
}

fn build_router(connection: DatabaseConnection) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(connection)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                let request_id: String = Uuid::new_v4().to_string();
                info_span!(
                    "http_request",
                    request_id = %request_id,
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version()
                )
            }),
        )
}

pub async fn get_connection_pool(configuration: &DatabaseSettings) -> DatabaseConnection {
    Database::connect(configuration.with_db())
        .await
        .expect("Failed to connect to database.")
}

pub struct Application {
    port: u16,
    server: Serve<TcpListener, Router, Router>,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        // 3️⃣ 建立数据库连接池
        // ⚠️ SeaORM 只有 async connect（没有 lazy）
        let db = get_connection_pool(&configuration.database).await;

        // 4️⃣ 绑定地址（不再硬编码）
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address).await?;
        let port = listener.local_addr()?.port();
        let server = run(listener, db);

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
