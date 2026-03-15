use migration::Migrator;
use sea_orm::{Database, DatabaseConnection, DbBackend, Statement};
use tokio::net::TcpListener;
use z2p_axum::configuration::DatabaseSettings;

use sea_orm_migration::MigratorTrait;
use z2p_axum::{configuration::get_configuration, startup::run};

use sea_orm::ConnectionTrait;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: sea_orm::DatabaseConnection,
}

pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

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
        db_pool: db,
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
