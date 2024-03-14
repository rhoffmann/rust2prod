use std::net::TcpListener;

use rust2prod::{configuration::get_configuration, startup::run};
use sqlx::PgPool;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_log::LogTracer;
use tracing::Subscriber;

// compose subscriber
pub fn get_subscriber(name: String, env_filter: String) -> impl Subscriber + Send + Sync {
    // set up bunyan logger with tracing-subscriber
    // fall back to printing all spans if RUST_LOG is not set
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new(env_filter));

    // output formatted logs to stdout
    let formatting_layer = BunyanFormattingLayer::new(name, std::io::stdout);

    // set up tracing subscriber and return it
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

// register once a subscriber as global default
pub fn init_subscriber_once(subscriber: impl Subscriber + Send + Sync) {
    // redirect all logs from the log crate to the tracing subscriber
    LogTracer::init().expect("Failed to set logger");

    // set subscriber as global default
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // configure tracing
    let subscriber = get_subscriber("rust2prod".into(), "info".into());
    init_subscriber_once(subscriber);

    // application code
    let configuration = get_configuration().expect("Failed to read configuration");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(&address).expect("Failed to bind port");

    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres.");

    println!("running on http://{}", &address);

    // need to await here, because we return a Server from run
    run(listener, connection_pool)?.await
}
