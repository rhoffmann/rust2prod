use actix_web::{http::header::ContentType, HttpRequest, HttpResponse};
use askama::Template;

use super::AuthLayout;

pub async fn login_form(request: HttpRequest) -> HttpResponse {
    // if the login page is openend directly with a flash cookie, we need to display it as an error message
    let error_html = match request.cookie("_flash") {
        None => "".into(),
        Some(cookie) => {
            let error = cookie.value();
            format!(
                include_str!("fragments/login_error.htmx.html"),
                htmlescape::encode_minimal(error)
            )
        }
    };

    let login = AuthLayout {
        title: "Login",
        error_message: &error_html,
        body: include_str!("login.html"),
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(login.render().unwrap())
}
