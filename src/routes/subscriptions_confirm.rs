use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirming a pending subscription", skip(_params))]
pub async fn confirm(_params: web::Query<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
