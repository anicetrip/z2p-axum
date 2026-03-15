pub mod entity;

use z2p_axum::{
    configuration::get_configuration,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // 1️⃣ telemetry —— 完全等价原书
    let subscriber = get_subscriber("z2p-axum".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // 2️⃣ 读取配置 —— 第四章核心
    let configuration = get_configuration().expect("Failed to read configuration.");

    let application = Application::build(configuration).await?;
    application.run_until_stopped().await
}
