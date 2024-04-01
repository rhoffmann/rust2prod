mod get;
mod post;

pub use get::login_form;
pub use post::login_post_fragment;

use askama::Template;

#[derive(Template)]
#[template(path = "auth.html", escape = "none")]
struct AuthLayout<'a> {
    title: &'a str,
    body: &'a str,
}
