use actix_web::HttpResponse;

// returns htmx fragment
pub async fn login_post_fragment() -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header(("HX-Redirect", "/"))
        // .insert_header((LOCATION, "/")) // this works only without htmx
        .finish()
    // HttpResponse::Ok().body(include_str!("fragments/login_success.htmx"))
}
