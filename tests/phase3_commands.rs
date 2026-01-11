use bytes::Bytes;
use padis::{Command, Frame};
use std::time::Duration;

// Helper to build a command frame
fn cmd_frame(args: &[&str]) -> Frame {
    Frame::Array(
        args.iter()
            .map(|s| Frame::BulkString(Bytes::copy_from_slice(s.as_bytes())))
            .collect(),
    )
}

// === PING ===

#[test]
fn parse_ping_no_args() {
    let frame = cmd_frame(&["PING"]);
    let cmd = Command::from_frame(frame).unwrap();
    assert!(matches!(cmd, Command::Ping { msg: None }));
}

#[test]
fn parse_ping_with_message() {
    let frame = cmd_frame(&["PING", "hello"]);
    let cmd = Command::from_frame(frame).unwrap();
    assert!(matches!(cmd, Command::Ping { msg: Some(m) } if m == "hello"));
}

#[test]
fn parse_ping_case_insensitive() {
    let frame = cmd_frame(&["ping"]);
    let cmd = Command::from_frame(frame).unwrap();
    assert!(matches!(cmd, Command::Ping { .. }));

    let frame = cmd_frame(&["PiNg"]);
    let cmd = Command::from_frame(frame).unwrap();
    assert!(matches!(cmd, Command::Ping { .. }));
}

// === ECHO ===

#[test]
fn parse_echo() {
    let frame = cmd_frame(&["ECHO", "hello world"]);
    let cmd = Command::from_frame(frame).unwrap();
    assert!(matches!(cmd, Command::Echo { msg } if msg == "hello world"));
}

#[test]
fn parse_echo_missing_arg() {
    let frame = cmd_frame(&["ECHO"]);
    let result = Command::from_frame(frame);
    assert!(result.is_err());
}

// === GET ===

#[test]
fn parse_get() {
    let frame = cmd_frame(&["GET", "mykey"]);
    let cmd = Command::from_frame(frame).unwrap();
    assert!(matches!(cmd, Command::Get { key } if key == "mykey"));
}

#[test]
fn parse_get_missing_key() {
    let frame = cmd_frame(&["GET"]);
    let result = Command::from_frame(frame);
    assert!(result.is_err());
}

#[test]
fn parse_get_too_many_args() {
    let frame = cmd_frame(&["GET", "key1", "key2"]);
    let result = Command::from_frame(frame);
    assert!(result.is_err());
}

// === SET ===

#[test]
fn parse_set_basic() {
    let frame = cmd_frame(&["SET", "mykey", "myvalue"]);
    let cmd = Command::from_frame(frame).unwrap();

    match cmd {
        Command::Set { key, value, expiry } => {
            assert_eq!(key, "mykey");
            assert_eq!(value, Bytes::from("myvalue"));
            assert!(expiry.is_none());
        }
        _ => panic!("expected Set command"),
    }
}

#[test]
fn parse_set_with_ex_seconds() {
    let frame = cmd_frame(&["SET", "mykey", "myvalue", "EX", "10"]);
    let cmd = Command::from_frame(frame).unwrap();

    match cmd {
        Command::Set { expiry, .. } => {
            assert_eq!(expiry, Some(Duration::from_secs(10)));
        }
        _ => panic!("expected Set command"),
    }
}

#[test]
fn parse_set_with_px_milliseconds() {
    let frame = cmd_frame(&["SET", "mykey", "myvalue", "PX", "1500"]);
    let cmd = Command::from_frame(frame).unwrap();

    match cmd {
        Command::Set { expiry, .. } => {
            assert_eq!(expiry, Some(Duration::from_millis(1500)));
        }
        _ => panic!("expected Set command"),
    }
}

#[test]
fn parse_set_ex_case_insensitive() {
    let frame = cmd_frame(&["set", "key", "val", "ex", "5"]);
    let cmd = Command::from_frame(frame).unwrap();

    match cmd {
        Command::Set { expiry, .. } => {
            assert_eq!(expiry, Some(Duration::from_secs(5)));
        }
        _ => panic!("expected Set command"),
    }
}

#[test]
fn parse_set_missing_value() {
    let frame = cmd_frame(&["SET", "mykey"]);
    let result = Command::from_frame(frame);
    assert!(result.is_err());
}

#[test]
fn parse_set_missing_expiry_value() {
    let frame = cmd_frame(&["SET", "mykey", "myvalue", "EX"]);
    let result = Command::from_frame(frame);
    assert!(result.is_err());
}

#[test]
fn parse_set_invalid_expiry_value() {
    let frame = cmd_frame(&["SET", "mykey", "myvalue", "EX", "notanumber"]);
    let result = Command::from_frame(frame);
    assert!(result.is_err());
}

// === Unknown Command ===

#[test]
fn parse_unknown_command() {
    let frame = cmd_frame(&["UNKNOWNCMD", "arg1"]);
    let result = Command::from_frame(frame);
    assert!(result.is_err());
}

// === Invalid Frame Types ===

#[test]
fn parse_non_array_frame_fails() {
    let frame = Frame::SimpleString("PING".to_string());
    let result = Command::from_frame(frame);
    assert!(result.is_err());
}

#[test]
fn parse_empty_array_fails() {
    let frame = Frame::Array(vec![]);
    let result = Command::from_frame(frame);
    assert!(result.is_err());
}

#[test]
fn parse_array_with_non_bulk_command_fails() {
    let frame = Frame::Array(vec![Frame::Integer(42)]);
    let result = Command::from_frame(frame);
    assert!(result.is_err());
}
