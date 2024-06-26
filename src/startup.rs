use std::net::TcpListener;

use crate::configuration::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use actix_files as fs;
use actix_web::cookie::Key;
use actix_web::{dev::Server, web, App, HttpServer};
use actix_web_flash_messages::{storage::CookieMessageStore, FlashMessagesFramework};

use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

use crate::routes::*;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");

        let timeout = configuration.email_client.timeout();

        let email_client = EmailClient::new(
            configuration.email_client.base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr().unwrap().port();

        let server = run(
            listener,
            connection_pool,
            email_client,
            configuration.application.base_url,
            configuration.application.hmac_secret,
        )?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configuration.with_db())
}

pub struct ApplicationBaseUrl(pub String);

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);

pub fn run(
    listener: TcpListener,
    connection_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>,
) -> Result<Server, std::io::Error> {
    let connection_pool = web::Data::new(connection_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    // let hmac_secret = web::Data::new(HmacSecret(hmac_secret);

    let message_store =
        CookieMessageStore::builder(Key::from(hmac_secret.expose_secret().as_bytes())).build();
    let message_framework = FlashMessagesFramework::builder(message_store).build();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(message_framework.clone())
            .wrap(TracingLogger::default())
            // pass in application state (needs to be cloned bc. each worker needs to have a copy)
            .app_data(connection_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
            // home
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login_post))
            // GET health_check
            .route("/health_check", web::get().to(health_check))
            // POST subscriptions
            .route("/subscriptions", web::post().to(subscribe))
            // POST subscriptions/confirm
            .route("/subscriptions/confirm", web::get().to(confirm))
            // GET subscriptions
            .route("/subscriptions", web::get().to(get_all_subscribers))
            // POST newsletters
            .route("/newsletters", web::post().to(publish_newsletter))
            // serve static files
            .service(
                fs::Files::new("/", "./static")
                    .use_last_modified(true)
                    .index_file("index.html"),
            )
    })
    .listen(listener)?
    .run();

    // error will be propagated from bind / listener
    // return server, no await
    Ok(server)
}
