mod get;
mod post;

pub use get::login_form;
pub use post::login_post;

use askama::Template;

#[derive(Template)]
#[template(path = "auth.html", escape = "none")]
struct AuthLayout<'a> {
    title: &'a str,
    body: &'a str,
}

#[derive(Template)]
#[template(path = "login/login.html", escape = "none")]
struct LoginPage<'a> {
    error_message: &'a str,
}
