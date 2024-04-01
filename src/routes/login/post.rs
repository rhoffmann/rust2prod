use actix_web::{web, HttpResponse};

use secrecy::Secret;
use sqlx::PgPool;

use crate::authentication::{validate_credentials, Credentials};

#[derive(serde::Deserialize)]
pub struct LoginData {
    pub email: String,
    pub password: Secret<String>,
}

// returns htmx fragment
#[tracing::instrument(
    name = "Login",
    skip(form, pool),
    fields(username=tracing::field::Empty, email=tracing::field::Empty)
)]
pub async fn login_post(form: web::Form<LoginData>, pool: web::Data<PgPool>) -> HttpResponse {
    let credentials = Credentials {
        username: form.0.email,
        password: form.0.password,
    };

    tracing::Span::current().record("username", &tracing::field::display(&credentials.username));

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            HttpResponse::SeeOther()
                // .insert_header((LOCATION, "/")) // use this if you want to redirect to the root path
                .insert_header(("HX-Redirect", "/"))
                .finish()
        }
        Err(_) => HttpResponse::Unauthorized().finish(), // TODO: add error message w/ htmx fragment OR redirect to login page / or just return 401
    }
}
