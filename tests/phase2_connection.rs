use bytes::Bytes;
use padis::{Connection, Frame};

#[tokio::test]
async fn read_single_frame() {
    let mock = tokio_test::io::Builder::new().read(b"+OK\r\n").build();

    let mut conn = Connection::new(mock);
    let frame = conn.read_frame().await.unwrap().unwrap();

    assert_eq!(frame, Frame::SimpleString("OK".to_string()));
}

#[tokio::test]
async fn read_frame_across_chunks() {
    // Frame arrives in three separate TCP packets
    let mock = tokio_test::io::Builder::new()
        .read(b"$5\r\n") // First packet: length header
        .read(b"hel") // Second packet: partial content
        .read(b"lo\r\n") // Third packet: rest of content
        .build();

    let mut conn = Connection::new(mock);
    let frame = conn.read_frame().await.unwrap().unwrap();

    assert_eq!(frame, Frame::BulkString(Bytes::from("hello")));
}

#[tokio::test]
async fn read_array_frame_chunked() {
    // A GET command split across packets
    let mock = tokio_test::io::Builder::new()
        .read(b"*2\r\n$3") // Array header + partial bulk
        .read(b"\r\nGET\r\n$3") // Rest of first element + partial second
        .read(b"\r\nfoo\r\n") // Rest of second element
        .build();

    let mut conn = Connection::new(mock);
    let frame = conn.read_frame().await.unwrap().unwrap();

    let expected = Frame::Array(vec![
        Frame::BulkString(Bytes::from("GET")),
        Frame::BulkString(Bytes::from("foo")),
    ]);
    assert_eq!(frame, expected);
}

#[tokio::test]
async fn read_multiple_frames_in_sequence() {
    let mock = tokio_test::io::Builder::new()
        .read(b"+OK\r\n:42\r\n") // Two frames in one packet
        .build();

    let mut conn = Connection::new(mock);

    let first = conn.read_frame().await.unwrap().unwrap();
    assert_eq!(first, Frame::SimpleString("OK".to_string()));

    let second = conn.read_frame().await.unwrap().unwrap();
    assert_eq!(second, Frame::Integer(42));
}

#[tokio::test]
async fn read_frame_eof_returns_none() {
    let mock = tokio_test::io::Builder::new().build(); // Empty - immediate EOF

    let mut conn = Connection::new(mock);
    let result = conn.read_frame().await.unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn read_frame_partial_then_eof_is_error() {
    let mock = tokio_test::io::Builder::new()
        .read(b"$5\r\nhel") // Incomplete frame, then EOF
        .build();

    let mut conn = Connection::new(mock);
    let result = conn.read_frame().await;

    assert!(result.is_err()); // Connection reset with partial frame = error
}

#[tokio::test]
async fn write_simple_string() {
    let mock = tokio_test::io::Builder::new().write(b"+OK\r\n").build();

    let mut conn = Connection::new(mock);
    conn.write_frame(&Frame::SimpleString("OK".to_string()))
        .await
        .unwrap();
}

#[tokio::test]
async fn write_bulk_string() {
    let mock = tokio_test::io::Builder::new()
        .write(b"$5\r\nhello\r\n")
        .build();

    let mut conn = Connection::new(mock);
    conn.write_frame(&Frame::BulkString(Bytes::from("hello")))
        .await
        .unwrap();
}

#[tokio::test]
async fn write_array() {
    let mock = tokio_test::io::Builder::new()
        .write(b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n")
        .build();

    let mut conn = Connection::new(mock);
    let frame = Frame::Array(vec![
        Frame::BulkString(Bytes::from("GET")),
        Frame::BulkString(Bytes::from("foo")),
    ]);
    conn.write_frame(&frame).await.unwrap();
}

#[tokio::test]
async fn write_null() {
    let mock = tokio_test::io::Builder::new().write(b"$-1\r\n").build();

    let mut conn = Connection::new(mock);
    conn.write_frame(&Frame::Null).await.unwrap();
}
