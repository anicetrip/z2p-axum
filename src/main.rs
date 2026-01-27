pub mod entity;
use sea_orm::Database;
// 新增这一行，声明 entity 模块
use tokio::net::TcpListener;
use z2p_axum::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_string = configuration.database.connection_string();

    let connection = Database::connect(&connection_string)
        .await
        .expect("Failed to connect to database.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).await.unwrap();
    // 一行启动服务器，与 Actix-web 版本结构一致
    run(listener, connection).await
}
