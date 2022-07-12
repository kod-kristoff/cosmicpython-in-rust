use std::net::TcpListener;

use axum::{extract::Extension, Router};

// use crate::routes;

pub async fn run(listener: TcpListener) -> std::io::Result<()> {
    println!("webapp::startup::run()");
    use axum::routing::{get, post};
    let app = Router::new();
    axum::Server::from_tcp(listener)
        .expect("Failed binding")
        .serve(app.into_make_service())
        .await
        .expect("Server error");
    Ok(())
}
