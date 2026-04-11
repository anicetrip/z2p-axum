use crate::email_client::EmailClient;
use crate::entity::subscription_tokens;
use crate::startup::AppState;
use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    entity::subscriptions,
};
use axum::{Form, extract::State, http::StatusCode};
use chrono::Utc;
use fake::RngExt;
use fake::rand::distr::Alphanumeric;
use fake::rand::rng;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, Set, TransactionTrait};
use serde::Deserialize;
use tracing;

#[derive(Deserialize, Debug)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, state),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    State(state): State<AppState>,
    Form(form): Form<FormData>,
) -> Result<StatusCode, StatusCode> {
    let new_subscriber: NewSubscriber = form.try_into().map_err(|_| StatusCode::BAD_REQUEST)?;

    let db = &state.db;
    // 2️⃣ 开启事务（SeaORM 方式）
    let txn = db
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 3️⃣ 插入 subscriber
    let subscriber_id = insert_subscriber(&txn, &new_subscriber)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    // 4️⃣ 生成 token
    let subscription_token = generate_subscription_token();

    // 5️⃣ 存 token
    store_token(&txn, subscriber_id, &subscription_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 6️⃣ 提交事务
    txn.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 7️⃣ 发邮件（事务之后）
    send_confirmation_email(
        &state.email_client,
        new_subscriber,
        &state.base_url,
        &subscription_token,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let _email_client = &state.email_client;
    let _base_url = &state.base_url;

    Ok(StatusCode::OK)
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, txn)
)]
pub async fn store_token(
    txn: &DatabaseTransaction,
    subscriber_id: i32,
    subscription_token: &str,
) -> Result<(), sea_orm::DbErr> {
    let token = subscription_tokens::ActiveModel {
        subscriber_id: Set(subscriber_id),
        subscription_token: Set(subscription_token.to_string()),
        ..Default::default()
    };

    token.insert(txn).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, db)
)]
pub async fn insert_subscriber(
    db: &DatabaseTransaction,
    new_subscriber: &NewSubscriber,
) -> Result<i32, sea_orm::DbErr> {
    let subscription = subscriptions::ActiveModel {
        // ❗ 不要设置 id
        email: Set(new_subscriber.email.as_ref().to_string()),
        name: Set(new_subscriber.name.as_ref().to_string()),
        subscribed_at: Set(Utc::now()),
        status: Set("pending_confirmation".to_string()),
        ..Default::default()
    };

    let result = subscription.insert(db).await?;

    Ok(result.id) // 👈 这里拿到数据库生成的 id
}

/// Generate a random 25-characters-long case-sensitive subscription token.
fn generate_subscription_token() -> String {
    let mut rng = rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
impl TryFrom<FormData> for NewSubscriber {
    type Error = String;
    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
}
