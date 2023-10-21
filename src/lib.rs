use serde::Deserialize;
use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpRequest, HttpResponse, HttpServer};

#[derive(Deserialize)]
struct SubscribeFormData {
    email: String,
    name: String,
}

async fn health_check(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn subscribe(form: web::Form<SubscribeFormData>) -> HttpResponse {
    let response = format!("subscribed name={}, email={}", form.email, form.name);
    HttpResponse::Ok().body(response)
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            // GET health_check
            .route("/health_check", web::get().to(health_check))
            // POST subscriptions
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    // error will be propagated from bind / listener
    // return server, no await
    Ok(server)
}
