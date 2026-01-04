// Implementation of a 'Frame' from the Redis serialisation protocol (RESP)
use bytes::Bytes;

// https://redis.io/docs/latest/develop/reference/protocol-spec/#resp-protocol-description
#[derive(Debug)]
pub enum Frame {
    SimpleString(String),
    SimpleError(String),
    Integer(u64),
    BulkString(Bytes),
    Array(Vec<Frame>),
}
