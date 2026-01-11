use crate::{Frame, frame::ParseError};
use bytes::{Buf, BytesMut};
use std::io::Cursor;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub struct Connection<T> {
    stream: T,
    buffer: BytesMut,
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Error while parsing")]
    Parse(#[from] ParseError),
    #[error("Connection reset with partial frame")]
    UnexpectedEof,
    #[error("Error while reading from the stream")]
    Read(#[from] std::io::Error),
}

// Implement some methods, where T is some type which implements these traits, for Connection<T>
impl<T: AsyncRead + AsyncWrite + Unpin> Connection<T> {
    pub fn new(stream: T) -> Self {
        Connection {
            stream,
            buffer: BytesMut::new(),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>, ConnectionError> {
        loop {
            let mut cursor = Cursor::new(&self.buffer[..]);
            match Frame::parse(&mut cursor) {
                Ok(frame) => {
                    self.buffer.advance(cursor.position() as usize);
                    return Ok(Some(frame));
                }
                Err(ParseError::Incomplete) => {
                    let n = self.stream.read_buf(&mut self.buffer).await?;
                    if n == 0 {
                        if self.buffer.is_empty() {
                            return Ok(None);
                        } else {
                            return Err(ConnectionError::UnexpectedEof);
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), ConnectionError> {
        let bytes = frame.to_bytes();
        self.stream.write_all(&bytes).await?;
        Ok(())
    }
}
