use actix_web::{http::header::ContentType, HttpResponse};
use askama::Template;

#[derive(Template)]
#[template(path = "app.html", escape = "none")]
struct AppLayout<'a> {
    title: &'a str,
    body: &'a str,
}

pub async fn home() -> HttpResponse {
    let home = AppLayout {
        title: "Homepage",
        body: include_str!("home.html"),
    };
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(home.render().unwrap())
}
