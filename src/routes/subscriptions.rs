use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    entity::subscription,
};
use axum::{Form, extract::State, http::StatusCode};
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
) -> Result<StatusCode, StatusCode> {
    let new_subscriber = form.try_into().map_err(|_| StatusCode::BAD_REQUEST)?;

    insert_subscriber(&db, new_subscriber)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, db)
)]
pub async fn insert_subscriber(
    db: &DatabaseConnection,
    new_subscriber: NewSubscriber,
) -> Result<(), sea_orm::DbErr> {
    let subscription = subscription::ActiveModel {
        email: ActiveValue::Set(new_subscriber.email.as_ref().to_string()),
        name: ActiveValue::Set(new_subscriber.name.as_ref().to_string()),
        subscribed_at: ActiveValue::Set(Utc::now()),
        ..Default::default()
    };

    subscription.insert(db).await?;

    Ok(())
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;
    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}
