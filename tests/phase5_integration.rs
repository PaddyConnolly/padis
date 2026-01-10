// tests/phase5_integration.rs
use padis::{Db, run_server};
use std::time::Duration;
use tokio::net::TcpListener;

async fn start_server() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let db = Db::new();

    tokio::spawn(async move {
        run_server(listener, db).await;
    });

    // Give server a moment to start
    tokio::time::sleep(Duration::from_millis(10)).await;
    port
}

fn connect(port: u16) -> redis::Client {
    redis::Client::open(format!("redis://127.0.0.1:{}", port)).unwrap()
}

#[tokio::test]
async fn ping_pong() {
    let port = start_server().await;
    let client = connect(port);
    let mut con = client.get_multiplexed_async_connection().await.unwrap();

    let result: String = redis::cmd("PING").query_async(&mut con).await.unwrap();

    assert_eq!(result, "PONG");
}

#[tokio::test]
async fn ping_with_message() {
    let port = start_server().await;
    let client = connect(port);
    let mut con = client.get_multiplexed_async_connection().await.unwrap();

    let result: String = redis::cmd("PING")
        .arg("hello")
        .query_async(&mut con)
        .await
        .unwrap();

    assert_eq!(result, "hello");
}

#[tokio::test]
async fn echo() {
    let port = start_server().await;
    let client = connect(port);
    let mut con = client.get_multiplexed_async_connection().await.unwrap();

    let result: String = redis::cmd("ECHO")
        .arg("hello world")
        .query_async(&mut con)
        .await
        .unwrap();

    assert_eq!(result, "hello world");
}

#[tokio::test]
async fn set_and_get() {
    let port = start_server().await;
    let client = connect(port);
    let mut con = client.get_multiplexed_async_connection().await.unwrap();

    let _: () = redis::cmd("SET")
        .arg("foo")
        .arg("bar")
        .query_async(&mut con)
        .await
        .unwrap();

    let result: String = redis::cmd("GET")
        .arg("foo")
        .query_async(&mut con)
        .await
        .unwrap();

    assert_eq!(result, "bar");
}

#[tokio::test]
async fn get_nonexistent_returns_nil() {
    let port = start_server().await;
    let client = connect(port);
    let mut con = client.get_multiplexed_async_connection().await.unwrap();

    let result: Option<String> = redis::cmd("GET")
        .arg("nonexistent")
        .query_async(&mut con)
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn set_with_expiry() {
    let port = start_server().await;
    let client = connect(port);
    let mut con = client.get_multiplexed_async_connection().await.unwrap();

    let _: () = redis::cmd("SET")
        .arg("expiring")
        .arg("value")
        .arg("PX")
        .arg("50")
        .query_async(&mut con)
        .await
        .unwrap();

    // Should exist immediately
    let result: Option<String> = redis::cmd("GET")
        .arg("expiring")
        .query_async(&mut con)
        .await
        .unwrap();
    assert_eq!(result, Some("value".to_string()));

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Should be gone
    let result: Option<String> = redis::cmd("GET")
        .arg("expiring")
        .query_async(&mut con)
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn multiple_clients_share_state() {
    let port = start_server().await;

    // Client 1 sets a value
    let client1 = connect(port);
    let mut con1 = client1.get_multiplexed_async_connection().await.unwrap();
    let _: () = redis::cmd("SET")
        .arg("shared")
        .arg("data")
        .query_async(&mut con1)
        .await
        .unwrap();

    // Client 2 reads the same value
    let client2 = connect(port);
    let mut con2 = client2.get_multiplexed_async_connection().await.unwrap();
    let result: String = redis::cmd("GET")
        .arg("shared")
        .query_async(&mut con2)
        .await
        .unwrap();

    assert_eq!(result, "data");
}

#[tokio::test]
async fn unknown_command_returns_error() {
    let port = start_server().await;
    let client = connect(port);
    let mut con = client.get_multiplexed_async_connection().await.unwrap();

    let result: Result<String, _> = redis::cmd("FOOBAR").query_async(&mut con).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn concurrent_clients() {
    let port = start_server().await;
    let mut handles = vec![];

    for i in 0..20 {
        let port = port;
        handles.push(tokio::spawn(async move {
            let client = connect(port);
            let mut con = client.get_multiplexed_async_connection().await.unwrap();

            let key = format!("key_{}", i);
            let value = format!("value_{}", i);

            let _: () = redis::cmd("SET")
                .arg(&key)
                .arg(&value)
                .query_async(&mut con)
                .await
                .unwrap();

            let result: String = redis::cmd("GET")
                .arg(&key)
                .query_async(&mut con)
                .await
                .unwrap();

            assert_eq!(result, value);
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}
