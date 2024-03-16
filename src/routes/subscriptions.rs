use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use crate::domain::*;

#[derive(Deserialize, Serialize)]
pub struct SubscribeFormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<SubscribeFormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {

    let name = match SubscriberName::parse(form.0.name) {
        Ok(name) => name,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let email = match SubscriberEmail::parse(form.0.email) {
        Ok(email) => email,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    let new_subscriber = NewSubscriber {
        email,
        name
    };

    match insert_subscriber(&pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, new_subscriber: &NewSubscriber) -> Result<(), sqlx::Error>{
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    Ok(())
}

pub async fn get_all_subscribers(_pool: web::Data<PgPool>) -> HttpResponse{
    HttpResponse::Ok().finish()
    // let query = sqlx::query!("SELECT email, name FROM subscriptions");
    // // let data: Vec<SubscribeFormData> = query.fetch_all(pool.get_ref()).await?.unwrap().collect();
    // match query.fetch_all(pool.get_ref()).await {
    //     Ok(result) => {
    //         let subscribers: Vec<SubscribeFormData> = result.collect();
    //         HttpResponse::Ok().json(subscribers)
    //     }
    //     Err(e) => {
    //         HttpResponse::InternalServerError().finish()
    //     }
    // }
}
