use tokio::net::TcpListener;
use z2p_axum::run;

use chrono::Utc;
use sea_orm::{EntityTrait, Set};
use uuid::Uuid;
use z2p_axum::entity::prelude::*;

use sea_orm::{Database};

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

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &address))
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
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
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

#[tokio::test]
async fn test_entity() {
    let active = SubscriptionsActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        email: Set("test@example.com".to_string()),
        name: Set("Test User".to_string()),
        subscribed_at: Set(Utc::now().into()),
    };

    println!("{:?}", active);
}




#[tokio::test]
async fn insert_subscription_works() {
    let db = Database::connect("mysql://root:1234@localhost:3306/newsletter")
        .await
        .expect("connect db");

    let id = Uuid::new_v4().to_string();
    let email = format!("test-{}@example.com", Uuid::new_v4());

    let active = SubscriptionsActiveModel {
        id: Set(id.clone()),
        email: Set(email.clone()),
        name: Set("Test User".to_string()),
        subscribed_at: Set(Utc::now().into()),
    };

    // ⚠️ 这里不要 expect
    let insert_result = SubscriptionsEntity::insert(active)
        .exec(&db)
        .await;

    match insert_result {
        Ok(_) => {} // PostgreSQL / SQLite 会走这里
        Err(sea_orm::DbErr::RecordNotInserted) => {
            // MySQL + 手动主键 = 正常现象
        }
        Err(e) => panic!("unexpected db error: {e:?}"),
    }

    // 再查一次，才是最终裁判
    let found = SubscriptionsEntity::find_by_id(id.clone())
        .one(&db)
        .await
        .expect("query subscription");

    assert!(found.is_some());

    let found = found.unwrap();
    assert_eq!(found.email, email);
}
