use padis::Frame;
use std::io::Cursor;
use std::string::ParseError;

#[test]
fn test_split_frame_parsing() {
    let mut raw = Cursor::new(&b"*2\r\n$3\r\nGET\r\n"[..]);

    // 1. Should fail with "Incomplete" because the second part of the array is missing
    let res = Frame::parse(&mut raw);
    assert!(matches!(res, Err(ParseError::Incomplete)));

    // 2. Append the rest and try again
    let mut raw_full = Cursor::new(&b"*2\r\n$3\r\nGET\r\n$3\r\nfoo\r\n"[..]);
    let res_full = Frame::parse(&mut raw_full).unwrap();

    // 3. Verify structure
    match res_full {
        Frame::Array(v) => assert_eq!(v.len(), 2),
        _ => panic!("Expected array"),
    }
}
