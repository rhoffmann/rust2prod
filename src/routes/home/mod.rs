use actix_web::{http::header::ContentType, HttpResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "app.html", escape = "none")]
struct AppLayout<'a> {
    title: &'a str,
    user: Option<String>,
    body: &'a str,
}

pub async fn home() -> HttpResponse {
    let home = AppLayout {
        title: "Homepage",
        user: None,
        body: include_str!("home.html"),
    };
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(home.render().unwrap())
}
