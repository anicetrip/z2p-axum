use migration::Migrator;
use sea_orm::{Database, DatabaseConnection, DbBackend, Statement};
use tokio::net::TcpListener;
use z2p_axum::configuration::DatabaseSettings;
use wiremock::MockServer;
use sea_orm_migration::MigratorTrait;
use z2p_axum::{configuration::get_configuration, startup::run};

use sea_orm::ConnectionTrait;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub email_server: MockServer,
    pub port: u16,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let email_server = MockServer::start().await;
    let mut configuration = get_configuration().expect("Failed to read configuration.");

    // 为每个测试生成唯一数据库名
    configuration.database.database_name = Uuid::new_v4().to_string();

    let db = configure_database(&configuration.database).await;

    let server = run(listener, db.clone());

    tokio::spawn(async move {
        server.await.expect("Failed to run server");
    });
    TestApp {
        address,
        email_server,
        port
    }
}

async fn configure_database(config: &DatabaseSettings) -> DatabaseConnection {
    // 1️⃣ 连接到 MySQL 实例（不指定数据库）
    let admin_conn = Database::connect(config.without_db())
        .await
        .expect("Failed to connect to MySQL without database");

    // 2️⃣ 创建测试数据库
    let create_db_stmt = Statement::from_string(
        DbBackend::MySql,
        format!(r#"CREATE DATABASE `{}`"#, config.database_name),
    );

    admin_conn
        .execute(create_db_stmt)
        .await
        .expect("Failed to create test database");

    // 3️⃣ 连接到新数据库
    let db_conn = Database::connect(config.with_db())
        .await
        .expect("Failed to connect to test database");

    // 4️⃣ 运行迁移
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
