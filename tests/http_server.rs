//! Integration tests for Zephyrite HTTP server

use std::time::Duration;

use reqwest::Client;
use serde_json::json;
use zephyrite::Config;
use zephyrite::server::Server;

/// Helper function to create a test server and return the client and server address
async fn setup_test_server() -> (
    Client,
    std::net::SocketAddr,
    tokio::sync::oneshot::Sender<()>,
) {
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let (addr_tx, addr_rx) = tokio::sync::oneshot::channel::<std::net::SocketAddr>();
    let config = Config::new(0); // Let OS pick a free port
    let server = Server::new(config).expect("Failed to create server");

    tokio::spawn(async move {
        server
            .start_with_shutdown(
                Some(async move {
                    shutdown_rx.await.ok();
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
    (client, addr, shutdown_tx)
}

#[tokio::test]
async fn health_check_works() {
    let (client, addr, shutdown_tx) = setup_test_server().await;

    let url = format!("http://{addr}/health");
    let resp = tokio::time::timeout(Duration::from_secs(2), client.get(&url).send())
        .await
        .expect("Request timed out")
        .expect("Failed to send request");

    assert!(resp.status().is_success());
    let json: serde_json::Value = resp.json().await.expect("Invalid JSON");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "Zephyrite");
    assert!(json["version"].is_string());

    let url = format!("http://{addr}/");
    let resp = tokio::time::timeout(Duration::from_secs(2), client.get(&url).send())
        .await
        .expect("Request timed out")
        .expect("Failed to send request");

    assert!(resp.status().is_success());
    let json: serde_json::Value = resp.json().await.expect("Invalid JSON");
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "Zephyrite");

    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn put_and_get_key_works() {
    let (client, addr, shutdown_tx) = setup_test_server().await;

    let key = "test_key";
    let value = "test_value";

    let put_url = format!("http://{addr}/keys/{key}");
    let put_body = json!({"value": value});
    let put_resp = tokio::time::timeout(
        Duration::from_secs(2),
        client.put(&put_url).json(&put_body).send(),
    )
    .await
    .expect("Request timed out")
    .expect("Failed to send request");

    assert_eq!(put_resp.status(), 201); // Created

    let get_url = format!("http://{addr}/keys/{key}");
    let get_resp = tokio::time::timeout(Duration::from_secs(2), client.get(&get_url).send())
        .await
        .expect("Request timed out")
        .expect("Failed to send request");

    assert!(get_resp.status().is_success());
    let json: serde_json::Value = get_resp.json().await.expect("Invalid JSON");
    assert_eq!(json["key"], key);
    assert_eq!(json["value"], value);
    assert_eq!(json["found"], true);
    assert!(json["size"].is_number());
    assert!(json["created_at"].is_string());
    assert!(json["updated_at"].is_string());

    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn update_existing_key_works() {
    let (client, addr, shutdown_tx) = setup_test_server().await;

    let key = "update_key";
    let initial_value = "initial_value";
    let updated_value = "updated_value";

    let put_url = format!("http://{addr}/keys/{key}");
    let put_body = json!({"value": initial_value});
    let put_resp = tokio::time::timeout(
        Duration::from_secs(2),
        client.put(&put_url).json(&put_body).send(),
    )
    .await
    .expect("Request timed out")
    .expect("Failed to send request");

    assert_eq!(put_resp.status(), 201); // Created

    let put_body = json!({"value": updated_value});
    let put_resp = tokio::time::timeout(
        Duration::from_secs(2),
        client.put(&put_url).json(&put_body).send(),
    )
    .await
    .expect("Request timed out")
    .expect("Failed to send request");

    assert_eq!(put_resp.status(), 200); // OK (updated)

    let get_url = format!("http://{addr}/keys/{key}");
    let get_resp = tokio::time::timeout(Duration::from_secs(2), client.get(&get_url).send())
        .await
        .expect("Request timed out")
        .expect("Failed to send request");

    assert!(get_resp.status().is_success());
    let json: serde_json::Value = get_resp.json().await.expect("Invalid JSON");
    assert_eq!(json["value"], updated_value);

    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn get_nonexistent_key_returns_404() {
    let (client, addr, shutdown_tx) = setup_test_server().await;

    let key = "nonexistent_key";
    let get_url = format!("http://{addr}/keys/{key}");
    let get_resp = tokio::time::timeout(Duration::from_secs(2), client.get(&get_url).send())
        .await
        .expect("Request timed out")
        .expect("Failed to send request");

    assert_eq!(get_resp.status(), 404);
    let json: serde_json::Value = get_resp.json().await.expect("Invalid JSON");
    assert_eq!(json["error"], "key_not_found");
    assert!(json["message"].as_str().unwrap().contains("not found"));

    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn delete_key_works() {
    let (client, addr, shutdown_tx) = setup_test_server().await;

    let key = "delete_key";
    let value = "delete_value";

    let put_url = format!("http://{addr}/keys/{key}");
    let put_body = json!({"value": value});
    let _put_resp = tokio::time::timeout(
        Duration::from_secs(2),
        client.put(&put_url).json(&put_body).send(),
    )
    .await
    .expect("Request timed out")
    .expect("Failed to send request");

    let delete_url = format!("http://{addr}/keys/{key}");
    let delete_resp =
        tokio::time::timeout(Duration::from_secs(2), client.delete(&delete_url).send())
            .await
            .expect("Request timed out")
            .expect("Failed to send request");

    assert_eq!(delete_resp.status(), 204); // No Content

    let get_url = format!("http://{addr}/keys/{key}");
    let get_resp = tokio::time::timeout(Duration::from_secs(2), client.get(&get_url).send())
        .await
        .expect("Request timed out")
        .expect("Failed to send request");

    assert_eq!(get_resp.status(), 404);

    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn delete_nonexistent_key_returns_404() {
    let (client, addr, shutdown_tx) = setup_test_server().await;

    let key = "nonexistent_delete_key";
    let delete_url = format!("http://{addr}/keys/{key}");
    let delete_resp =
        tokio::time::timeout(Duration::from_secs(2), client.delete(&delete_url).send())
            .await
            .expect("Request timed out")
            .expect("Failed to send request");

    assert_eq!(delete_resp.status(), 404); // Not Found

    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn list_keys_works() {
    let (client, addr, shutdown_tx) = setup_test_server().await;

    let list_url = format!("http://{addr}/keys");
    let list_resp = tokio::time::timeout(Duration::from_secs(2), client.get(&list_url).send())
        .await
        .expect("Request timed out")
        .expect("Failed to send request");

    assert!(list_resp.status().is_success());
    let json: serde_json::Value = list_resp.json().await.expect("Invalid JSON");
    assert_eq!(json["count"], 0);
    assert_eq!(json["keys"].as_array().unwrap().len(), 0);

    let keys_values = vec![("key1", "value1"), ("key2", "value2"), ("key3", "value3")];

    for (key, value) in &keys_values {
        let put_url = format!("http://{addr}/keys/{key}");
        let put_body = json!({"value": value});
        let _put_resp = tokio::time::timeout(
            Duration::from_secs(2),
            client.put(&put_url).json(&put_body).send(),
        )
        .await
        .expect("Request timed out")
        .expect("Failed to send request");
    }

    let list_resp = tokio::time::timeout(Duration::from_secs(2), client.get(&list_url).send())
        .await
        .expect("Request timed out")
        .expect("Failed to send request");

    assert!(list_resp.status().is_success());
    let json: serde_json::Value = list_resp.json().await.expect("Invalid JSON");
    assert_eq!(json["count"], 3);

    let returned_keys: Vec<String> = json["keys"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    assert_eq!(returned_keys.len(), 3);
    for (key, _) in &keys_values {
        assert!(returned_keys.contains(&(*key).to_string()));
    }

    let _ = shutdown_tx.send(());
}

#[tokio::test]
async fn invalid_json_returns_400() {
    let (client, addr, shutdown_tx) = setup_test_server().await;

    let key = "test_key";
    let put_url = format!("http://{addr}/keys/{key}");

    let put_resp = tokio::time::timeout(
        Duration::from_secs(2),
        client
            .put(&put_url)
            .header("Content-Type", "application/json")
            .body("invalid json")
            .send(),
    )
    .await
    .expect("Request timed out")
    .expect("Failed to send request");

    assert_eq!(put_resp.status(), 400); // Bad Request

    let _ = shutdown_tx.send(());
}
