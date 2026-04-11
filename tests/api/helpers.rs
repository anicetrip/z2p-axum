use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use wiremock::MockServer;
use z2p_axum::configuration::DatabaseSettings;
use z2p_axum::configuration::get_configuration;

use migration::{Migrator, MigratorTrait};
use uuid::Uuid;
use z2p_axum::startup::{Application, get_connection_pool};

pub struct TestApp {
    pub address: String,
    pub email_server: MockServer,
    pub db_pool: DatabaseConnection,
    pub port: u16,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub async fn spawn_app() -> TestApp {
    dotenvy::dotenv().ok();
    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri().parse().unwrap();
        c
    };
    configure_database(&configuration.database).await;
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let application_port = application.port();
    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database).await,
        email_server,
        port: application_port,
    }
}

async fn configure_database(config: &DatabaseSettings) -> DatabaseConnection {
    let admin_conn = Database::connect(config.without_db())
        .await
        .expect("Failed to connect to MySQL without database");

    // 👇 关键：用 execute_unprepared（DDL必须）
    admin_conn
        .execute_unprepared(&format!(r#"CREATE DATABASE `{}`"#, config.database_name))
        .await
        .expect("Failed to create test database");

    let db_conn = Database::connect(config.with_db())
        .await
        .expect("Failed to connect to test database");

    Migrator::up(&db_conn, None)
        .await
        .expect("Failed to run migrations");

    db_conn
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        // Extract the link from one of the request fields.
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // Let's make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };
        let html = get_link(&body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(&body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, plain_text }
    }
}
