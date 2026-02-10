pub mod entity;

use sea_orm::Database;
use tokio::net::TcpListener;

use z2p_axum::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1️⃣ telemetry —— 完全等价原书
    let subscriber = get_subscriber("z2p-axum".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // 2️⃣ 读取配置 —— 第四章核心
    let configuration = get_configuration().expect("Failed to read configuration.");

    // 3️⃣ 建立数据库连接池
    // ⚠️ SeaORM 只有 async connect（没有 lazy）
    let db = Database::connect(configuration.database.with_db())
        .await
        .expect("Failed to connect to database.");

    // 4️⃣ 绑定地址（不再硬编码）
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address).await?;

    // 5️⃣ 启动 axum（结构上等价 actix）
    run(listener, db).await
}
