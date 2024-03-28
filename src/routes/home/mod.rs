use actix_web::HttpResponse;
use askama::Template;

#[derive(Template)]
#[template(path = "app.html", escape = "none")]
struct AppLayout<'a> {
    title: &'a str,
    body: &'a str,
}

#[derive(Template)]
#[template(path = "auth.html", escape = "none")]
struct AuthLayout<'a> {
    title: &'a str,
    body: &'a str,
}

pub async fn home() -> HttpResponse {
    let home = AppLayout {
        title: "Homepage",
        body: include_str!("home.html"),
    };
    HttpResponse::Ok().body(home.render().unwrap())
}

// returns htmx fragment
pub async fn login() -> HttpResponse {
    let login = AuthLayout {
        title: "Login",
        body: include_str!("login.html"),
    };
    HttpResponse::Ok().body(login.render().unwrap())
}

// returns htmx fragment
pub async fn post_login() -> HttpResponse {
    HttpResponse::Ok().body(include_str!("fragments/login_success.htmx"))
}
