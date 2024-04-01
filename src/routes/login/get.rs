use actix_web::{http::header::ContentType, HttpResponse};
use askama::Template;

use super::AuthLayout;

pub async fn login_form() -> HttpResponse {
    let login = AuthLayout {
        title: "Login",
        body: include_str!("login.html"),
    };
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(login.render().unwrap())
}
