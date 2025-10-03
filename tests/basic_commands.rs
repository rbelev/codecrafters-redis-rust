mod common;

use common::*;

#[tokio::test]
async fn test_ping_command() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    let response = client
        .send_array(&["PING"])
        .await
        .expect("Failed to send PING");
    assert_eq!(parse_simple_string(&response), Some("PONG"));
}

#[tokio::test]
async fn test_ping_with_message() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    let response = client
        .send_array(&["PING", "Hello"])
        .await
        .expect("Failed to send PING");
    assert_eq!(parse_bulk_string(&response), Some("Hello".to_string()));
}

#[tokio::test]
async fn test_echo_command() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    let response = client
        .send_array(&["ECHO", "test message"])
        .await
        .expect("Failed to send ECHO");
    assert_eq!(
        parse_bulk_string(&response),
        Some("test message".to_string())
    );
}

#[tokio::test]
async fn test_set_and_get() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    // SET command
    let response = client
        .send_array(&["SET", "key1", "value1"])
        .await
        .expect("Failed to send SET");
    assert_eq!(parse_simple_string(&response), Some("OK"));

    // GET command
    let response = client
        .send_array(&["GET", "key1"])
        .await
        .expect("Failed to send GET");
    assert_eq!(parse_bulk_string(&response), Some("value1".to_string()));
}

#[tokio::test]
async fn test_get_nonexistent_key() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    let response = client
        .send_array(&["GET", "nonexistent"])
        .await
        .expect("Failed to send GET");
    assert!(response.starts_with("$-1")); // Null bulk string
}

#[tokio::test]
async fn test_multiple_clients() {
    let server = TestServer::start().await.expect("Failed to start server");

    let mut client1 = server.connect().await.expect("Failed to connect client 1");
    let mut client2 = server.connect().await.expect("Failed to connect client 2");

    // Client 1 sets a value
    client1
        .send_array(&["SET", "shared", "data"])
        .await
        .expect("Failed to SET");

    // Client 2 reads it
    let response = client2
        .send_array(&["GET", "shared"])
        .await
        .expect("Failed to GET");
    assert_eq!(parse_bulk_string(&response), Some("data".to_string()));
}
