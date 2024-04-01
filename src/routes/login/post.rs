use actix_web::{web, HttpResponse};

use secrecy::Secret;

#[derive(serde::Deserialize)]
pub struct LoginData {
    pub email: String,
    pub password: Secret<String>,
}

// returns htmx fragment
pub async fn login_post_fragment(_form: web::Form<LoginData>) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header(("HX-Redirect", "/"))
        // .insert_header((LOCATION, "/")) // this works only without htmx
        .finish()
    // HttpResponse::Ok().body(include_str!("fragments/login_success.htmx"))
}
