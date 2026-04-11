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
use crate::email_client::EmailClient;
use crate::routes::confirm;
use crate::{
    configuration::Settings,
    routes::{health_check, subscribe},
};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub email_client: EmailClient,
    pub base_url: String,
}

pub fn run(
    listener: TcpListener,
    connection: DatabaseConnection,
    email_client: EmailClient,
    base_url: String,
) -> Serve<TcpListener, Router, Router> {
    let state = AppState {
        db: connection,
        email_client,
        base_url,
    };

    let router = build_router(state);
    axum::serve(listener, router)
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .with_state(state)
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
        let connection_pool = get_connection_pool(&configuration.database).await;
        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address).await?;
        let port = listener.local_addr()?.port();
        let server = run(
            listener,
            connection_pool,
            email_client,
            configuration.application.base_url,
        );

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
