mod common;

use common::*;

#[tokio::test]
#[ignore]
async fn test_rpush_single_element() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    let response = client
        .send_array(&["RPUSH", "mylist", "element1"])
        .await
        .expect("Failed to send RPUSH");
    assert_eq!(parse_integer(&response), Some(1));
}

#[tokio::test]
#[ignore]
async fn test_rpush_multiple_elements() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    let response = client
        .send_array(&["RPUSH", "mylist", "a", "b", "c"])
        .await
        .expect("Failed to send RPUSH");
    assert_eq!(parse_integer(&response), Some(3));
}

#[tokio::test]
#[ignore]
async fn test_lpush_single_element() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    let response = client
        .send_array(&["LPUSH", "mylist", "element1"])
        .await
        .expect("Failed to send LPUSH");
    assert_eq!(parse_integer(&response), Some(1));
}

#[tokio::test]
#[ignore]
async fn test_lrange_command() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    // Setup: Add elements
    client
        .send_array(&["RPUSH", "mylist", "one", "two", "three"])
        .await
        .expect("Failed to RPUSH");

    // Test LRANGE
    let response = client
        .send_array(&["LRANGE", "mylist", "0", "-1"])
        .await
        .expect("Failed to send LRANGE");
    let elements = parse_array(&response).expect("Failed to parse array");

    assert_eq!(elements, vec!["one", "two", "three"]);
}

#[tokio::test]
#[ignore]
async fn test_lrange_partial() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    client
        .send_array(&["RPUSH", "mylist", "a", "b", "c", "d", "e"])
        .await
        .expect("Failed to RPUSH");

    let response = client
        .send_array(&["LRANGE", "mylist", "1", "3"])
        .await
        .expect("Failed to send LRANGE");
    let elements = parse_array(&response).expect("Failed to parse array");

    assert_eq!(elements, vec!["b", "c", "d"]);
}

#[tokio::test]
#[ignore]
async fn test_rpush_then_lpush() {
    let server = TestServer::start().await.expect("Failed to start server");
    let mut client = server.connect().await.expect("Failed to connect");

    client
        .send_array(&["RPUSH", "mylist", "middle"])
        .await
        .expect("Failed to RPUSH");
    client
        .send_array(&["LPUSH", "mylist", "first"])
        .await
        .expect("Failed to LPUSH");
    client
        .send_array(&["RPUSH", "mylist", "last"])
        .await
        .expect("Failed to RPUSH");

    let response = client
        .send_array(&["LRANGE", "mylist", "0", "-1"])
        .await
        .expect("Failed to LRANGE");
    let elements = parse_array(&response).expect("Failed to parse array");

    assert_eq!(elements, vec!["first", "middle", "last"]);
}
