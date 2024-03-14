use actix_web::{web, HttpResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use tracing::Instrument;

#[derive(Deserialize, Serialize)]
pub struct SubscribeFormData {
    email: String,
    name: String,
}


pub async fn subscribe(
    form: web::Form<SubscribeFormData>,
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // let new_user_str = format!("name='{}', email='{}'", form.email, form.name);
    let trace_id = Uuid::new_v4();
    let request_span = tracing::info_span!("Adding a new subscriber", %trace_id, subscriber_email = %form.email, subscriber_name = %form.name);

    let _request_span_guard = request_span.enter();

    let query = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    );

    let _query_span = tracing::info_span!("Saving new subscriber details in database");

    match query.execute(pool.get_ref()).await {
        Ok(_) => {
            tracing::info!("New subscriber successfully saved");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
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