// Implementation of a 'Frame' from the Redis serialisation protocol (RESP)
use atoi::atoi;
use bytes::{Buf, Bytes};
use std::{io::Cursor, string::FromUtf8Error};

// https://redis.io/docs/latest/develop/reference/protocol-spec/#resp-protocol-description
#[derive(Debug, Clone)]
pub enum Frame {
    SimpleString(String),
    SimpleError(String), // Not a code error, Redis responding with an Error
    Integer(i64),
    BulkString(Bytes),
    Null,
    Array(Vec<Frame>),
}

#[derive(Debug)]
pub enum ParseError {
    Incomplete,
    InvalidInteger,
    InvalidString,
    InvalidEnd,
    Other(crate::Error),
}

impl Frame {
    // Create an empty array
    pub fn array() -> Frame {
        Frame::Array(vec![])
    }

    // Parse RESP2 data from a buffer
    pub fn parse(buf: &mut Cursor<&[u8]>) -> Result<Frame, ParseError> {
        match get_u8(buf)? {
            b'+' => {
                // SimpleString
                let line = get_line(buf)?.to_vec();

                let string = String::from_utf8(line)?;

                Ok(Frame::SimpleString(string))
            }
            b'-' => {
                // SimpleError
                let line = get_line(buf)?.to_vec();

                let string = String::from_utf8(line)?;

                Ok(Frame::SimpleError(string))
            }
            b':' => {
                // Integer
                let line = get_line(buf)?.to_vec();

                let integer = get_integer(&line)?;

                Ok(Frame::Integer(integer))
            }
            b'$' => {
                // BulkString
                // If not null
                if peek_u8(buf)? != b'-' {
                    let line = get_line(buf)?.to_vec();
                    let len = (get_integer(&line)? + 2) as usize;

                    let data = Bytes::copy_from_slice(&buf.chunk()[..len]);

                    buf.advance(len);

                    Ok(Frame::BulkString(data))

                // If null
                } else {
                    Ok(Frame::Null)
                }
            }
            b'*' => {
                // Array
                let line = get_line(buf)?.to_vec();
                let len = (get_integer(&line)? + 2) as usize;
                let mut out = Vec::with_capacity(len);

                for _ in 0..len {
                    out.push(Frame::parse(buf)?);
                }

                Ok(Frame::Array(out))
            }
            _ => {
                // If char not in RESP2, panic with unimplemented
                unimplemented!()
            }
        }
    }
}

// Get a char from the buffer
fn get_u8(buf: &mut Cursor<&[u8]>) -> Result<u8, ParseError> {
    if !buf.has_remaining() {
        return Err(ParseError::Incomplete);
    }

    Ok(buf.get_u8())
}

// Peek at the next char from the buffer
fn peek_u8(buf: &mut Cursor<&[u8]>) -> Result<u8, ParseError> {
    if !buf.has_remaining() {
        return Err(ParseError::Incomplete);
    }

    Ok(buf.chunk()[0])
}

// Get a line from the buffer
fn get_line<'a>(buf: &mut Cursor<&'a [u8]>) -> Result<&'a [u8], ParseError> {
    let start = buf.position() as usize;

    let end = buf.get_ref().len() - 1;

    for i in start..end {
        if buf.get_ref()[i] == b'\r' && buf.get_ref()[i + 1] == b'\n' {
            buf.set_position((i + 2) as u64);
            return Ok(&buf.get_ref()[start..i]);
        }
    }

    Err(ParseError::Incomplete)
}

// Get an integer from a line
fn get_integer(line: &[u8]) -> Result<i64, ParseError> {
    let nums = strip_line(line)?;

    match atoi::<i64>(nums) {
        Some(n) => Ok(n),
        _ => Err(ParseError::InvalidInteger),
    }
}

// Strip CRLF from a line
fn strip_line(line: &[u8]) -> Result<&[u8], ParseError> {
    if line.len() < 2 || &line[line.len() - 2..] != b"\r\n" {
        return Err(ParseError::InvalidEnd);
    }

    Ok(&line[..line.len() - 2])
}

//  Implement the trait From, to enable conversions from &str to ParseError
impl From<&str> for ParseError {
    fn from(src: &str) -> ParseError {
        ParseError::Other(src.into())
    }
}

//  Implement the trait From, to enable conversions from FromUtf8Error to ParseError
impl From<FromUtf8Error> for ParseError {
    fn from(_src: FromUtf8Error) -> ParseError {
        "Parse Error: Invalid frame format".into()
    }
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Frame::SimpleString(a), Frame::SimpleString(b)) => a == b,
            (Frame::SimpleError(a), Frame::SimpleError(b)) => a == b,
            (Frame::Integer(a), Frame::Integer(b)) => a == b,
            (Frame::BulkString(a), Frame::BulkString(b)) => a == b,
            (Frame::Array(a), Frame::Array(b)) => a == b,
            _ => false,
        }
    }
}
