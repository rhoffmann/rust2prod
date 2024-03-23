use actix_web::{web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum ConfirmError {
    #[error("Could not find subscriber with that token")]
    UnauthorizedError(),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub struct ConfirmSubscriberError(sqlx::Error);

impl std::fmt::Debug for ConfirmSubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(&self, f)
    }
}

impl std::fmt::Display for ConfirmSubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to confirm subscriber")
    }
}

impl std::error::Error for ConfirmSubscriberError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[tracing::instrument(name = "Confirming a pending subscription", skip(params, pool))]
pub async fn confirm(
    params: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfirmError> {
    let id = get_subscriber_id_by_token(&pool, &params.subscription_token)
        .await
        .context("Failed to retrieve subscriber ID by token")?;

    match id {
        None => return Err(ConfirmError::UnauthorizedError()),
        Some(subscriber_id) => {
            confirm_subscriber(&pool, subscriber_id)
                .await
                .context("Failed to confirm subscriber")?;
            Ok(HttpResponse::Ok().finish())
        }
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(pool, subscriber_id))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        UPDATE subscriptions
        SET status = 'confirmed'
        WHERE id = $1
        "#,
        subscriber_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[tracing::instrument(
    name = "Fetching subscriber ID from the database",
    skip(pool, subscribtion_token)
)]
pub async fn get_subscriber_id_by_token(
    pool: &PgPool,
    subscribtion_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT subscriber_id
        FROM subscription_tokens
        WHERE subscription_token = $1
        "#,
        subscribtion_token
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.subscriber_id))
}
