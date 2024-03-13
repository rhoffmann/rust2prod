use std::net::TcpListener;

use actix_web::{dev::Server, middleware::Logger, web, App, HttpServer};
use sqlx::PgPool;

use crate::routes::*;

pub fn run(listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    let connection_pool = web::Data::new(connection_pool);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            // pass in application state (needs to be cloned bc. each worker needs to have a copy)
            .app_data(connection_pool.clone())
            .route("/greet", web::get().to(greet))
            .route("/greet/{name}", web::get().to(greet))
            // GET health_check
            .route("/health_check", web::get().to(health_check))
            // POST subscriptions
            .route("/subscriptions", web::post().to(subscribe))
            // GET subscriptions
            .route("/subscriptions", web::get().to(get_all_subscribers))
    })
    .listen(listener)?
    .run();

    // error will be propagated from bind / listener
    // return server, no await
    Ok(server)
}
