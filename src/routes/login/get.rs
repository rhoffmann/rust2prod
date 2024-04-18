use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::{IncomingFlashMessages, Level};
use askama::Template;
use std::fmt::Write;

use super::{AuthLayout, LoginPage};

pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    // if the login page is openend directly with a flash cookie, we need to display it as an error message
    let mut error_html = String::new();

    for message in flash_messages.iter().filter(|m| m.level() == Level::Error) {
        writeln!(error_html, "<p>{}</p>", message.content()).unwrap();
    }

    let error_html = format!(
        include_str!("fragments/login_error.htmx.html"),
        htmlescape::encode_minimal(error_html.as_str())
    );

    let page = LoginPage {
        error_message: &error_html,
    };

    let login = AuthLayout {
        title: "Login",
        body: &page.render().unwrap(),
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(login.render().unwrap())
}
