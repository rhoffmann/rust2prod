use std::net::TcpListener;
use secrecy::ExposeSecret;

use rust2prod::{configuration::get_configuration, startup::run};
use sqlx::PgPool;
use rust2prod::telemetry::{get_subscriber, init_subscriber_once};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // configure tracing
    let subscriber = get_subscriber("rust2prod".into(), "info".into(), std::io::stdout);
    init_subscriber_once(subscriber);

    // application code
    let configuration = get_configuration().expect("Failed to read configuration");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(&address).expect("Failed to bind port");

    let connection_pool = PgPool::connect(&configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to postgres.");

    println!("running on http://{}", &address);

    // need to await here, because we return a Server from run
    run(listener, connection_pool)?.await
}
