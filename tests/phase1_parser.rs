// tests/phase1_parser.rs
use bytes::Bytes;
use padis::frame::{Frame, ParseError};
use std::io::Cursor;

#[test]
fn parse_simple_string() {
    let mut cursor = Cursor::new(&b"+OK\r\n"[..]);
    let frame = Frame::parse(&mut cursor).unwrap();
    assert_eq!(frame, Frame::SimpleString("OK".to_string()));
}

#[test]
fn parse_error() {
    let mut cursor = Cursor::new(&b"-ERR unknown command\r\n"[..]);
    let frame = Frame::parse(&mut cursor).unwrap();
    assert_eq!(frame, Frame::SimpleError("ERR unknown command".to_string()));
}

#[test]
fn parse_integer() {
    let mut cursor = Cursor::new(&b":42\r\n"[..]);
    let frame = Frame::parse(&mut cursor).unwrap();
    assert_eq!(frame, Frame::Integer(42));
}

#[test]
fn parse_negative_integer() {
    let mut cursor = Cursor::new(&b":-1\r\n"[..]);
    let frame = Frame::parse(&mut cursor).unwrap();
    assert_eq!(frame, Frame::Integer(-1));
}

#[test]
fn parse_bulk_string() {
    let mut cursor = Cursor::new(&b"$5\r\nhello\r\n"[..]);
    let frame = Frame::parse(&mut cursor).unwrap();
    assert_eq!(frame, Frame::BulkString(Bytes::from("hello")));
}

#[test]
fn parse_empty_bulk_string() {
    let mut cursor = Cursor::new(&b"$0\r\n\r\n"[..]);
    let frame = Frame::parse(&mut cursor).unwrap();
    assert_eq!(frame, Frame::BulkString(Bytes::from("")));
}

#[test]
fn parse_null() {
    let mut cursor = Cursor::new(&b"$-1\r\n"[..]);
    let frame = Frame::parse(&mut cursor).unwrap();
    assert_eq!(frame, Frame::Null);
}

#[test]
fn parse_array() {
    let mut cursor = Cursor::new(&b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n"[..]);
    let frame = Frame::parse(&mut cursor).unwrap();

    let expected = Frame::Array(vec![
        Frame::BulkString(Bytes::from("GET")),
        Frame::BulkString(Bytes::from("foo")),
    ]);
    assert_eq!(frame, expected);
}

#[test]
fn parse_empty_array() {
    let mut cursor = Cursor::new(&b"*0\r\n"[..]);
    let frame = Frame::parse(&mut cursor).unwrap();
    assert_eq!(frame, Frame::Array(vec![]));
}

#[test]
fn incomplete_simple_string() {
    let mut cursor = Cursor::new(&b"+OK"[..]); // missing \r\n
    let result = Frame::parse(&mut cursor);
    assert!(matches!(result, Err(ParseError::Incomplete)));
}

#[test]
fn incomplete_bulk_string_header() {
    let mut cursor = Cursor::new(&b"$5\r\nhel"[..]); // missing bytes + \r\n
    let result = Frame::parse(&mut cursor);
    assert!(matches!(result, Err(ParseError::Incomplete)));
}

#[test]
fn incomplete_array_partial_element() {
    // Array says 2 elements, but only 1 is present
    let mut cursor = Cursor::new(&b"*2\r\n$3\r\nGET\r\n"[..]);
    let result = Frame::parse(&mut cursor);
    assert!(matches!(result, Err(ParseError::Incomplete)));
}

#[test]
fn invalid_frame_type() {
    let mut cursor = Cursor::new(&b"^invalid\r\n"[..]);
    let result = Frame::parse(&mut cursor);
    assert!(matches!(result, Err(ParseError::Incomplete)));
}

#[test]
fn cursor_advances_past_frame() {
    let mut cursor = Cursor::new(&b"+OK\r\n+NEXT\r\n"[..]);

    let _ = Frame::parse(&mut cursor).unwrap();
    // Cursor should now be at position 5, ready to parse "+NEXT\r\n"
    assert_eq!(cursor.position(), 5);

    let second = Frame::parse(&mut cursor).unwrap();
    assert_eq!(second, Frame::SimpleString("NEXT".to_string()));
}
