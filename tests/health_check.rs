use tokio::net::TcpListener;
use z2p_axum::run;

pub async fn spawn_app() -> String {
    // 1. 绑定到随机端口
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");

    // 2. 获取实际端口
    let port = listener.local_addr().unwrap().port();

    // 3. 启动服务器（后台运行）
    tokio::spawn(async move {
        run(listener).await.expect("服务器运行失败");
    });

    // 4. 返回完整的地址（例如 "http://127.0.0.1:54321"）
    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

}
