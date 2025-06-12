//! Integration tests for Zephyrite HTTP server

use std::time::Duration;

use reqwest::Client;
use zephyrite::Config;
use zephyrite::server::Server;

#[tokio::test]
async fn health_check_works() {
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let (addr_tx, addr_rx) = tokio::sync::oneshot::channel::<std::net::SocketAddr>();
    let config = Config::new(0); // Let OS pick a free port
    let server = Server::new(config);

    let server_task = tokio::spawn(async move {
        server
            .start_with_shutdown(
                Some(async move {
                    shutdown_rx.await.ok();
                    println!("[server] Shutdown signal received");
                }),
                Some(addr_tx),
            )
            .await
    });

    let addr = tokio::time::timeout(Duration::from_secs(2), addr_rx)
        .await
        .expect("Timed out waiting for server address")
        .expect("Server failed to send address");

    let client = Client::new();
    let url = format!("http://{addr}/health");
    let resp = tokio::time::timeout(Duration::from_secs(2), client.get(&url).send())
        .await
        .expect("Request timed out")
        .expect("Failed to send request");

    assert!(resp.status().is_success());
    let json: serde_json::Value = resp.json().await.expect("Invalid JSON");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "Zephyrite");

    let _ = shutdown_tx.send(());

    tokio::time::timeout(Duration::from_secs(2), server_task)
        .await
        .expect("Server shutdown timed out")
        .expect("Server task join error")
        .expect("Server returned error");
}
