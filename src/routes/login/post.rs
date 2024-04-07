use actix_web::{error::InternalError, http::header, http::header::ContentType, web, HttpResponse};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    errors::error_chain_fmt,
    startup::HmacSecret,
};

#[derive(serde::Deserialize)]
pub struct LoginData {
    pub email: String,
    pub password: Secret<String>,
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Unexpected error occurred")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

// returns htmx fragment
#[tracing::instrument(
    name = "Login",
    skip(form, pool, secret),
    fields(username=tracing::field::Empty, email=tracing::field::Empty)
)]
pub async fn login_post(
    form: web::Form<LoginData>,
    pool: web::Data<PgPool>,
    secret: web::Data<HmacSecret>,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credentials = Credentials {
        username: form.0.email,
        password: form.0.password,
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

            // user is authenticated, redirect to home page
            Ok(HttpResponse::SeeOther()
                .insert_header(("HX-Redirect", "/"))
                .finish())
        }
        Err(e) => {
            let (error_message, e) = match e {
                AuthError::InvalidCredentials(_) => (
                    "Invalid credentials".to_string(),
                    LoginError::AuthError(e.into()),
                ),
                AuthError::UnexpectedError(_) => (
                    "An unexpected error occurred".to_string(),
                    LoginError::UnexpectedError(e.into()),
                ),
            };

            let hmac_tag = {
                let mut mac =
                    Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes())
                        .unwrap();
                mac.update(error_message.as_bytes());
                mac.finalize().into_bytes()
            };

            let response = match e {
                // invalid credentials, show error message in the fragment
                LoginError::AuthError(_) => {
                    // simple pass-in error message to the fragment (could go for askama template here as well)
                    let response_fragment = format!(
                        include_str!("fragments/login_error.htmx.html"),
                        error_message
                    );
                    // 422 unprocessable entity would be more appropriate, but we want to show the error message
                    // 418 I'm a teapot is a fun status code to use for this purpose
                    // htmx needs to be configured to allow successfull swap for the response type
                    HttpResponse::ImATeapot()
                        .insert_header(("hmac-tag", format!("{hmac_tag:x}")))
                        .insert_header(("error-message", error_message))
                        .content_type(ContentType::html())
                        .body(response_fragment)
                }
                // unexpected error, redirect to login page
                LoginError::UnexpectedError(_) => HttpResponse::SeeOther()
                    .insert_header((header::LOCATION, "/login"))
                    .insert_header(("HX-Redirect", "/"))
                    .insert_header(("hmac-tag", format!("{hmac_tag:x}")))
                    .insert_header(("error-message", error_message))
                    .finish(),
            };

            Err(InternalError::from_response(e, response))
        }
    }
}
