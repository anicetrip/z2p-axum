

use tokio::net::TcpListener;
use z2p_axum::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    // 一行启动服务器，与 Actix-web 版本结构一致
    run(listener).await
}
