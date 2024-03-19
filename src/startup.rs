use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use actix_files as fs;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use crate::configuration::Settings;
use crate::email_client::EmailClient;

use crate::routes::*;


pub async fn build(configuration: Settings) -> Result<Server, std::io::Error> {
    let connection_pool = PgPool::connect_lazy_with(configuration.database.with_db());
    let sender_email = configuration.email_client.sender().expect("Invalid sender email address");
    let timeout = configuration.email_client.timeout();

    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(&address)?;

    run(listener, connection_pool, email_client)
}

pub fn run(listener: TcpListener, connection_pool: PgPool, email_client: EmailClient) -> Result<Server, std::io::Error> {
    let connection_pool = web::Data::new(connection_pool);
    let email_client = web::Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            // pass in application state (needs to be cloned bc. each worker needs to have a copy)
            .app_data(connection_pool.clone())
            .app_data(email_client.clone())
            // .route("/{filename:.*}", web::get().to(static_files))
            .route("/greet", web::get().to(greet))
            .route("/greet/{name}", web::get().to(greet))
            // GET health_check
            .route("/health_check", web::get().to(health_check))
            // POST subscriptions
            .route("/subscriptions", web::post().to(subscribe))
            // GET subscriptions
            .route("/subscriptions", web::get().to(get_all_subscribers))
            // serve static files
            .service(fs::Files::new("/", "./static").use_last_modified(true).index_file("index.html"))
    })
        .listen(listener)?
        .run();

    // error will be propagated from bind / listener
    // return server, no await
    Ok(server)
}
