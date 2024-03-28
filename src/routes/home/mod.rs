use actix_web::HttpResponse;
use askama::Template;

#[derive(Template)]
#[template(path = "app.html", escape = "none")]
struct LayoutTemplate<'a> {
    title: &'a str,
    body: &'a str,
}

pub async fn home() -> HttpResponse {
    let home = LayoutTemplate {
        title: "Homepage",
        body: include_str!("home.html"),
    };
    HttpResponse::Ok().body(home.render().unwrap())
}

// returns html fragment
pub async fn login() -> HttpResponse {
    HttpResponse::Ok().body(include_str!("fragments/login_success.htmx"))
}
