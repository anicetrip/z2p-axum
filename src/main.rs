pub mod entity;
use sea_orm::Database;
use secrecy::ExposeSecret;
// 新增这一行，声明 entity 模块
use tokio::net::TcpListener;
use z2p_axum::{configuration::get_configuration, startup::run, telemetry::{get_subscriber, init_subscriber}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let subscriber = get_subscriber(
        "z2p-axum".into(),
        "info".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_string = configuration
        .database
        .connection_string()
        .expose_secret()
        .to_owned();

    let connection = Database::connect(&connection_string)
        .await
        .expect("Failed to connect to database.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address).await.unwrap();
    // 一行启动服务器，与 Actix-web 版本结构一致
    run(listener, connection).await
}
