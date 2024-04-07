use actix_web::{error::InternalError, http::header::ContentType, web, HttpResponse};
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

            Ok(HttpResponse::SeeOther()
                .insert_header(("HX-Redirect", "/"))
                .finish())
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };

            let error_message = match e {
                LoginError::AuthError(_) => "Invalid credentials".to_string(),
                LoginError::UnexpectedError(_) => "An unexpected error occurred".to_string(),
            };

            let hmac_tag = {
                let mut mac =
                    Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes())
                        .unwrap();
                mac.update(error_message.as_bytes());
                mac.finalize().into_bytes()
            };

            // simple pass-in error message to the fragment (could go for askama template here as well)
            let response_fragment = format!(
                include_str!("fragments/login_error.htmx.html"),
                error_message
            );

            // diverging from the book here, we return a 200 OK response with the error message in the body
            let response = HttpResponse::Ok()
                .insert_header(("hmac-tag", format!("{hmac_tag:x}")))
                .insert_header(("error-message", error_message))
                .content_type(ContentType::html())
                .body(response_fragment);

            Err(InternalError::from_response(e, response))
        }
    }
}
