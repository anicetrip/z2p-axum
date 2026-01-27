use crate::entity::subscription;
use axum::Form;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use chrono::Utc;
use sea_orm::ActiveValue::NotSet;
use sea_orm::ActiveValue::Set;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)] // 添加 Debug 便于日志打印
pub struct FormData {
    pub email: String,
    pub name: String,
}

pub async fn subscribe(
    State(db): State<DatabaseConnection>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    println!("Name: {}, Email: {}", form.name, form.email);

    let uuid = Uuid::new_v4().to_string();

    println!("UUID: {}", uuid);
    println!("UUID: {}, Length: {}", uuid, uuid.len());

    let subscription = subscription::ActiveModel {
        email: Set(form.email.clone()),
        name: Set(form.name.clone()),
        subscribed_at: Set(Utc::now()),
        id: NotSet, // 手动设置时间
    };

    match subscription::Entity::insert(subscription).exec(&db).await {
        Ok(result) => {
            println!("Insert result: {:?}", result);
            (StatusCode::OK, "Subscribed!").into_response()
        }
        Err(e) => {
            println!("Failed to execute query: {:?}", e);
            // 打印更详细的错误信息
            eprintln!("Detailed error: {:#?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to subscribe").into_response()
        }
    }
}
