use crate::Frame;
use bytes::Bytes;
use std::time::Duration;

pub enum Command {
    Ping {
        msg: Option<Bytes>,
    },
    Echo {
        msg: Bytes,
    },
    Get {
        key: Bytes,
    },
    Set {
        key: Bytes,
        value: Bytes,
        expiry: Option<Duration>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Expected array for command parsing")]
    ExpectedArray,
    #[error("Unknown Command")]
    Unknown(String),
    #[error("Expected command, got empty frame")]
    Empty,
    #[error("Failed to parse command name")]
    InvalidCommandName,
    #[error("Expected String for message")]
    InvalidMsg,
    #[error("Invalid expiry")]
    InvalidExpiry,
}

impl Command {
    pub fn from_frame(frame: Frame) -> Result<Command, CommandError> {
        match frame {
            Frame::Array(mut frames) => {
                if frames.is_empty() {
                    return Err(CommandError::Empty);
                }

                let cmd = match frames.remove(0) {
                    Frame::BulkString(b) => b,
                    _ => return Err(CommandError::InvalidCommandName),
                };

                match cmd.to_ascii_uppercase().as_slice() {
                    b"GET" => parse_get(&frames),
                    b"SET" => parse_set(&frames),
                    b"PING" => Ok(parse_ping(&frames)),
                    b"ECHO" => parse_echo(&frames),
                    _ => Err(CommandError::Unknown(String::from_utf8_lossy(&cmd).into())),
                }
            }
            _ => Err(CommandError::ExpectedArray),
        }
    }
}

fn parse_get(frames: &[Frame]) -> Result<Command, CommandError> {
    if let [Frame::BulkString(key)] = frames {
        Ok(Command::Get { key: key.clone() })
    } else {
        Err(CommandError::InvalidMsg)
    }
}

fn parse_set(frames: &[Frame]) -> Result<Command, CommandError> {
    if let [Frame::BulkString(key), Frame::BulkString(value), rest @ ..] = frames {
        let expiry = match rest {
            [] => None,
            [Frame::BulkString(unit), Frame::BulkString(time)] => {
                let time = std::str::from_utf8(time)
                    .map_err(|_| CommandError::InvalidExpiry)?
                    .parse::<u64>()
                    .map_err(|_| CommandError::InvalidExpiry)?;
                match unit.to_ascii_lowercase().as_slice() {
                    b"ex" => Some(Duration::from_secs(time)),
                    b"px" => Some(Duration::from_millis(time)),
                    _ => return Err(CommandError::InvalidExpiry),
                }
            }
            _ => return Err(CommandError::InvalidExpiry),
        };
        Ok(Command::Set {
            key: key.clone(),
            value: value.clone(),
            expiry,
        })
    } else {
        Err(CommandError::InvalidMsg)
    }
}

fn parse_ping(frames: &[Frame]) -> Command {
    if let [Frame::BulkString(msg), ..] = frames {
        Command::Ping {
            msg: Some(msg.clone()),
        }
    } else {
        Command::Ping { msg: None }
    }
}
fn parse_echo(frames: &[Frame]) -> Result<Command, CommandError> {
    if let Some(Frame::BulkString(msg)) = frames.first() {
        Ok(Command::Echo { msg: msg.clone() })
    } else {
        Err(CommandError::InvalidMsg)
    }
}
