use std::net::TcpListener;
use webapp::startup;

#[tokio::test]
async fn api_returns_allocation() {
    let app = spawn_app().await;
}

pub struct TestApp {
    pub address: String,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    tokio::spawn(async move { startup::run(listener).await });

    TestApp { address }
}
