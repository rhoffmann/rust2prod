use actix_web::{
    error::InternalError,
    http::header::{self, ContentType},
    web, HttpResponse,
};
use actix_web_flash_messages::FlashMessage;
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
    #[error("Invalid credentials")]
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
    skip(form, pool),
    fields(username=tracing::field::Empty, email=tracing::field::Empty)
)]
pub async fn login_post(
    form: web::Form<LoginData>,
    pool: web::Data<PgPool>,
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
            // TODO: is this the right way to handle the error? looks a bit convoluted
            // do we really need the LoginError enum or can we directly use the AuthError?
            // probably that dance is not necessary when we use htmx?
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

            FlashMessage::error(&error_message).send();

            let response = match e {
                // invalid credentials, show error message in the fragment
                LoginError::AuthError(_) => {
                    // simple pass-in error message to the fragment (could go for askama template here as well)
                    let response_fragment = format!(
                        include_str!("fragments/login_error.htmx.html"),
                        &error_message
                    );
                    // 422 unprocessable entity would be more appropriate, but we want to show the error message
                    // 418 I'm a teapot is a fun status code to use for this purpose
                    // htmx needs to be configured to allow successfull swap for the response type
                    HttpResponse::ImATeapot()
                        .content_type(ContentType::html())
                        .body(response_fragment)
                }

                // unexpected error, redirect to login page
                LoginError::UnexpectedError(_) => HttpResponse::SeeOther()
                    .insert_header((header::LOCATION, "/login"))
                    .insert_header(("HX-Redirect", "/"))
                    .finish(),
            };

            Err(InternalError::from_response(e, response))
        }
    }
}
