mod common;

use common::*;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_set_with_expiry_px() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    // Set with 100ms expiry
    client
        .send_array(&["SET", "tempkey", "tempvalue", "PX", "100"])
        .await
        .expect("Failed to SET");

    // Should exist immediately
    let response = client
        .send_array(&["GET", "tempkey"])
        .await
        .expect("Failed to GET");
    assert_eq!(parse_bulk_string(&response), Some("tempvalue".to_string()));

    // Wait for expiry
    sleep(Duration::from_millis(150)).await;

    // Should be gone
    let response = client
        .send_array(&["GET", "tempkey"])
        .await
        .expect("Failed to GET");
    assert!(response.starts_with("$-1"));
}

#[tokio::test]
async fn test_set_with_expiry_ex() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    // Set with 1 second expiry
    client
        .send_array(&["SET", "tempkey", "tempvalue", "EX", "1"])
        .await
        .expect("Failed to SET");

    let response = client
        .send_array(&["GET", "tempkey"])
        .await
        .expect("Failed to GET");
    assert_eq!(parse_bulk_string(&response), Some("tempvalue".to_string()));

    sleep(Duration::from_secs(2)).await;

    let response = client
        .send_array(&["GET", "tempkey"])
        .await
        .expect("Failed to GET");
    assert!(response.starts_with("$-1"));
}
