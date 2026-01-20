use z2p_axum::run;

fn spawn_app() {
    // 注意：run() 是 async fn，返回 Future
    let future = run(); // 获取 Future，但代码还未执行

    // 将 Future 交给 Tokio 在后台运行
    let _ = tokio::spawn(async {
        future.await.expect("服务器运行失败");
    });
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    spawn_app();

    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
