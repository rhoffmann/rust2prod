use actix_web::HttpResponse;
use askama::Template;

#[derive(Template)]
#[template(path = "layout.html", escape = "none")]
struct LayoutTemplate<'a> {
    title: &'a str,
    body: &'a str,
}

pub async fn home() -> HttpResponse {
    let home = LayoutTemplate {
        title: "Home",
        body: include_str!("home.html"),
    };
    HttpResponse::Ok().body(home.render().unwrap())
}
