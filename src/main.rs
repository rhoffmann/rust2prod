use std::net::TcpListener;

use rust2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind port");
    let socket = listener.local_addr().unwrap();
    let port = socket.port();
    let ip = socket.ip().to_string();
    println!("running on http://{}:{}", ip, port);

    run(listener)?.await
}
