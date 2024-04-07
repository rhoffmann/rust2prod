use actix_web::{http::header::ContentType, web, HttpResponse};
use askama::Template;
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;

use crate::startup::HmacSecret;

use super::AuthLayout;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));

        let mut mac =
            Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;

        Ok(self.error)
    }
}

pub async fn login_form(
    query: Option<web::Query<QueryParams>>,
    secret: web::Data<HmacSecret>,
) -> HttpResponse {
    let login = AuthLayout {
        title: "Login",
        body: include_str!("login.html"),
    };

    // IF we get a query parameter, we know that the login failed (but we use htmx to show the error message)
    // we currently don't use the tag for anything, but we could use it to prevent CSRF attacks
    let _error_html = match query {
        None => "".into(),
        Some(query) => match query.0.verify(&secret) {
            Ok(error) => {
                format!(
                    include_str!("fragments/login_error.htmx.html"),
                    htmlescape::encode_minimal(&error)
                )
            }
            Err(e) => {
                tracing::warn!("Failed to verify query parameter: {:?}", e);
                "".into()
            }
        },
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(login.render().unwrap())
}
