use std::net::TcpListener;

use env_logger::Env;
use rust2prod::{configuration::get_configuration, startup::run};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(&address).expect("Failed to bind port");

    // set_logger will be called by init
    // fall back to level 'info' when RUST_LOG is not set
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres.");

    println!("running on http://{}", &address);

    run(listener, connection_pool)?.await
}
