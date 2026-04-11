use crate::startup::AppState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use sea_orm::{DatabaseConnection, RelationTrait};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, state))]
pub async fn confirm(
    State(state): State<AppState>,
    Query(parameters): Query<Parameters>,
) -> Result<StatusCode, StatusCode> {
    let id = match get_subscriber_id_from_token(&state.db, &parameters.subscription_token).await {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    match id {
        // Non-existing token
        None => Err(StatusCode::UNAUTHORIZED),

        Some(subscriber_id) => {
            if confirm_subscriber(&state.db, subscriber_id).await.is_err() {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            Ok(StatusCode::OK)
        }
    }
}

use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect};

use crate::entity::{subscription_tokens, subscriptions};
#[tracing::instrument(name = "Get subscriber_id from token", skip(db, token))]
pub async fn get_subscriber_id_from_token(
    db: &DatabaseConnection,
    token: &str,
) -> Result<Option<i32>, DbErr> {
    let result = subscription_tokens::Entity::find()
        // 👉 JOIN subscriptions
        .join(
            JoinType::InnerJoin,
            subscription_tokens::Relation::Subscriptions.def(),
        )
        // 👉 WHERE token
        .filter(subscription_tokens::Column::SubscriptionToken.eq(token))
        // 👉 只选 subscriber_id（可选优化）
        .select_only()
        .column(subscription_tokens::Column::SubscriberId)
        .into_tuple::<i32>() // 👈 关键
        .one(db)
        .await?;

    Ok(result)
}

use sea_orm::{ActiveModelTrait, DbErr, Set};

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(db, subscriber_id))]
pub async fn confirm_subscriber(db: &DatabaseConnection, subscriber_id: i32) -> Result<(), DbErr> {
    // 1️⃣ 查找 subscriber
    let subscriber = subscriptions::Entity::find_by_id(subscriber_id)
        .one(db)
        .await?
        .ok_or_else(|| DbErr::Custom("Subscriber not found".into()))?;

    // 2️⃣ 转 ActiveModel
    let mut subscriber: subscriptions::ActiveModel = subscriber.into();

    // 3️⃣ 修改状态
    subscriber.status = Set("confirmed".to_string());

    // 4️⃣ 更新
    subscriber.update(db).await?;

    Ok(())
}
