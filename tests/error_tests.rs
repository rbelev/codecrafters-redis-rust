mod common;

use common::*;

#[tokio::test]
async fn test_wrong_number_of_arguments() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    // SET requires at least 2 arguments
    let response = client
        .send_array(&["SET", "key"])
        .await
        .expect("Failed to send SET");
    assert!(response.starts_with("-ERR"));
}

#[tokio::test]
async fn test_unknown_command() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    let response = client
        .send_array(&["UNKNOWN", "arg"])
        .await
        .expect("Failed to send command");
    assert!(response.starts_with("-ERR"));
}

#[tokio::test]
async fn test_wrong_type_operation() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    // Set a string value
    client
        .send_array(&["SET", "mykey", "string_value"])
        .await
        .expect("Failed to SET");

    // Try to use list operation on it
    let response = client
        .send_array(&["RPUSH", "mykey", "element"])
        .await
        .expect("Failed to RPUSH");
    assert!(response.starts_with("-WRONGTYPE") || response.starts_with("-ERR"));
}
