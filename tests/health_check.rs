use migration::Migrator;
use sea_orm_migration::MigratorTrait;
use secrecy::ExposeSecret;
use tokio::net::TcpListener;
use z2p_axum::{
    configuration::{DatabaseSettings, get_configuration},
    startup::run,
};

use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use uuid::Uuid;

use sea_orm::Database;

pub struct TestApp {
    pub address: String,
    pub db_pool: sea_orm::DatabaseConnection,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    // 为每个测试生成唯一的数据库名称
    configuration.database.database_name = Uuid::new_v4().to_string();
    print!("DB Name: {}\n", configuration.database.database_name);

    // 配置数据库（创建测试数据库）
    let connection_pool = configure_database(&configuration.database).await;

    // 启动服务器
    let server = run(listener, connection_pool.clone());

    // 在后台运行服务器
    tokio::spawn(async move {
        server.await.expect("Failed to run server");
    });

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> DatabaseConnection {
    // 1. 连接到 MySQL 实例（不指定数据库）
    let root_url = format!(
        "mysql://{}:{}@{}:{}",
        config.username,
        config.password.expose_secret(),
        config.host,
        config.port
    );

    // 使用 root 连接创建数据库
    let admin_conn = Database::connect(&root_url)
        .await
        .expect("Failed to connect to MySQL as admin");

    // 2. 创建数据库
    let create_db_query = format!("CREATE DATABASE IF NOT EXISTS `{}`", config.database_name);
    admin_conn
        .execute(Statement::from_string(DbBackend::MySql, create_db_query))
        .await
        .expect("Failed to create database.");

    // 3. 连接到新创建的数据库
    let database_url = config.connection_string().expose_secret().to_owned();
    let connection = Database::connect(&database_url)
        .await
        .expect("Failed to connect to database.");

    // 4. 运行 SeaORM 迁移
    <Migrator as MigratorTrait>::up(&connection, None)
        .await
        .expect("Failed to run migrations");

    connection
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            422,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
