use actix_web::{http::header::ContentType, web, HttpResponse, ResponseError};
use hmac::{Hmac, Mac};
use reqwest::StatusCode;
use secrecy::Secret;
use sqlx::PgPool;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials},
    errors::error_chain_fmt,
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

impl ResponseError for LoginError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::AuthError(_) => StatusCode::OK, // or StatusCode::UNAUTHORIZED when serving full pages
            Self::UnexpectedError(_) => StatusCode::OK, // or StatusCode::INTERNAL_SERVER_ERROR when serving full pages
        }
    }

    fn error_response(&self) -> HttpResponse {
        let secret: &[u8] = "super secret!".as_bytes();
        let error_message = match self {
            Self::AuthError(_) => "Invalid credentials".to_string(),
            Self::UnexpectedError(_) => "Unexpected error occurred".to_string(),
        };

        let hmac_tag = {
            let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret).unwrap();
            mac.update(error_message.as_bytes());
            mac.finalize().into_bytes()
        };

        let response_fragment = format!(
            include_str!("fragments/login_error.htmx.html"),
            error_message
        );

        HttpResponse::build(self.status_code())
            .insert_header(("hmac-tag", format!("{hmac_tag:x}")))
            .insert_header(("error-message", error_message))
            .content_type(ContentType::html())
            .body(response_fragment)
    }
}

// returns htmx fragment
#[tracing::instrument(
    name = "Login",
    skip(form, pool),
    fields(username=tracing::field::Empty, email=tracing::field::Empty)
)]
pub async fn login_post(
    form: web::Form<LoginData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, LoginError> {
    let credentials = Credentials {
        username: form.0.email,
        password: form.0.password,
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    let user_id = validate_credentials(credentials, &pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
            AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
        })?;

    tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

    Ok(HttpResponse::SeeOther()
        .insert_header(("HX-Redirect", "/"))
        .finish())
}
