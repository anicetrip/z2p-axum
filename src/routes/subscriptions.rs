use axum::{
    Form,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};
use serde::Deserialize;
use tracing;

#[derive(Deserialize, Debug)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    State(db): State<DatabaseConnection>,
    Form(form): Form<FormData>,
) -> Response {
    match insert_subscriber(&db, &form).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[tracing::instrument(name = "Saving new subscriber details in the database", skip(form, db))]
pub async fn insert_subscriber(
    db: &DatabaseConnection,
    form: &FormData,
) -> Result<(), sea_orm::DbErr> {
    use crate::entity::subscription;

    // 创建 ActiveModel - id 不设置，让数据库自增生成
    let subscription = subscription::ActiveModel {
        email: ActiveValue::Set(form.email.clone()),
        name: ActiveValue::Set(form.name.clone()),
        subscribed_at: ActiveValue::Set(Utc::now()),
        ..Default::default() // id 会使用默认值（数据库自增）
    };

    // 现在可以用 insert() 了，因为 id 是自增的
    subscription.insert(db).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
