use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct NewsletterBody {
    title: String,
    content: NewsletterContent,
}

#[derive(serde::Deserialize)]
pub struct NewsletterContent {
    html: String,
    text: String,
}

pub async fn publish_newsletter(_body: web::Json<NewsletterBody>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
