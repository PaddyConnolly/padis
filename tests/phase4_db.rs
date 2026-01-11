use bytes::Bytes;
use padis::Db;
use std::time::Duration;

#[test]
fn get_nonexistent_key() {
    let db = Db::new();
    let result = db.get(&Bytes::from("missing"));
    assert!(result.is_none());
}

#[test]
fn set_and_get() {
    let db = Db::new();
    db.set(&Bytes::from("key"), Bytes::from("value"), None);
    let result = db.get(&Bytes::from("key")).unwrap();
    assert_eq!(result, Bytes::from("value"));
}

#[test]
fn set_overwrites_existing() {
    let db = Db::new();
    db.set(&Bytes::from("key"), Bytes::from("first"), None);
    db.set(&Bytes::from("key"), Bytes::from("second"), None);
    let result = db.get(&Bytes::from("key")).unwrap();
    assert_eq!(result, Bytes::from("second"));
}

#[test]
fn del_existing_key() {
    let db = Db::new();
    db.set(&Bytes::from("key"), Bytes::from("value"), None);
    let deleted = db.del(&Bytes::from("key"));
    assert!(deleted);
    assert!(db.get(&Bytes::from("key")).is_none());
}

#[test]
// tests/phase3_commands.rs
fn del_nonexistent_key() {
    let db = Db::new();
    let deleted = db.del(&Bytes::from("missing"));
    assert!(!deleted);
}

#[test]
fn clone_shares_state() {
    let db1 = Db::new();
    let db2 = db1.clone();
    db1.set(&Bytes::from("key"), Bytes::from("value"), None);
    // db2 should see the same data
    let result = db2.get(&Bytes::from("key")).unwrap();
    assert_eq!(result, Bytes::from("value"));
}

#[tokio::test]
async fn expired_key_returns_none() {
    let db = Db::new();
    db.set(
        &Bytes::from("key"),
        Bytes::from("value"),
        Some(Duration::from_millis(50)),
    );
    // Key should exist immediately
    assert!(db.get(&Bytes::from("key")).is_some());
    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(100)).await;
    // Key should be gone
    assert!(db.get(&Bytes::from("key")).is_none());
}

#[tokio::test]
async fn set_with_expiry_then_overwrite_without_expiry() {
    let db = Db::new();
    // Set with short expiry
    db.set(
        &Bytes::from("key"),
        Bytes::from("v1"),
        Some(Duration::from_millis(50)),
    );
    // Overwrite without expiry
    db.set(&Bytes::from("key"), Bytes::from("v2"), None);
    // Wait past original expiry
    tokio::time::sleep(Duration::from_millis(100)).await;
    // Key should still exist (expiry was cleared)
    let result = db.get(&Bytes::from("key")).unwrap();
    assert_eq!(result, Bytes::from("v2"));
}

#[tokio::test]
async fn concurrent_writes_no_data_loss() {
    let db = Db::new();
    let mut handles = vec![];
    for i in 0..100 {
        let db_clone = db.clone();
        handles.push(tokio::spawn(async move {
            db_clone.set(
                &Bytes::from(format!("key_{}", i)),
                Bytes::from("value"),
                None,
            );
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }
    // All 100 keys should exist
    for i in 0..100 {
        assert!(
            db.get(&Bytes::from(format!("key_{}", i))).is_some(),
            "Missing key_{}",
            i
        );
    }
}

#[tokio::test]
async fn concurrent_read_write_same_key() {
    let db = Db::new();
    db.set(&Bytes::from("counter"), Bytes::from("0"), None);
    let mut handles = vec![];
    // Spawn readers and writers competing for the same key
    for i in 0..50 {
        let db_clone = db.clone();
        handles.push(tokio::spawn(async move {
            db_clone.set(&Bytes::from("counter"), Bytes::from(format!("{}", i)), None);
        }));
        let db_clone = db.clone();
        handles.push(tokio::spawn(async move {
            let _ = db_clone.get(&Bytes::from("counter")); // Just read, don't panic
        }));
    }
    for handle in handles {
        handle.await.unwrap();
    }
    // Key should exist with some value (we don't care which writer won)
    assert!(db.get(&Bytes::from("counter")).is_some());
}

#[test]
fn keys_returns_all_keys() {
    let db = Db::new();
    db.set(&Bytes::from("a"), Bytes::from("1"), None);
    db.set(&Bytes::from("b"), Bytes::from("2"), None);
    db.set(&Bytes::from("c"), Bytes::from("3"), None);
    let mut keys = db.keys();
    keys.sort();
    assert_eq!(
        keys,
        vec![Bytes::from("a"), Bytes::from("b"), Bytes::from("c")]
    );
}
