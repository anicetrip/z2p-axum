use z2p_axum::run;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 一行启动服务器，与 Actix-web 版本结构一致
    run().await
} 