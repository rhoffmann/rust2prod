use std::net::TcpListener;

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

    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(&address).expect("Failed to bind port");

    let connection_pool = PgPool::connect_lazy_with(configuration.database.with_db());

    println!("running on http://{}", &address);

    // need to await here, because we return a Server from run
    run(listener, connection_pool)?.await
}
