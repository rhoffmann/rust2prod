use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let socket = listener.local_addr().unwrap();
    let port = socket.port();
    let ip = socket.ip().to_string();

    let server = rust2prod::run(listener).expect("Failed to bind address");

    // launch server as background task
    // drop the spawned future handle
    let _ = tokio::spawn(server);

    format!("http://{}:{}", ip, port)
}
