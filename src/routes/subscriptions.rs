use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SubscribeFormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<SubscribeFormData>) -> HttpResponse {
    let response = format!("subscribed name={}, email={}", form.email, form.name);
    HttpResponse::Ok().body(response)
}
