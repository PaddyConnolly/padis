use crate::{Command, Connection, Db, Frame};
use tokio::net::{TcpListener, TcpStream};

pub async fn run_server(listener: TcpListener, db: Db) {
    loop {
        match listener.accept().await {
            Ok((socket, _addr)) => {
                let db = db.clone();
                tokio::spawn(async move {
                    handle_connection(socket, db).await;
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        }
    }
}

async fn handle_connection(socket: TcpStream, db: Db) {
    let mut conn = Connection::new(socket);

    loop {
        let frame = match conn.read_frame().await {
            Ok(Some(frame)) => frame,
            Ok(None) => {
                return;
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
                return;
            }
        };

        let response = match Command::from_frame(frame) {
            Ok(cmd) => execute(cmd, &db),
            Err(e) => Frame::SimpleError(e.to_string()),
        };

        if let Err(e) = conn.write_frame(&response).await {
            eprintln!("Failed to write response: {}", e);
            return;
        }
    }
}

fn execute(cmd: Command, db: &Db) -> Frame {
    match cmd {
        Command::Ping { msg } => match msg {
            None => Frame::SimpleString("PONG".into()),
            Some(m) => Frame::BulkString(m),
        },
        Command::Echo { msg } => Frame::BulkString(msg),
        Command::Get { key } => match db.get(&key) {
            Some(v) => Frame::BulkString(v),
            _ => Frame::Null,
        },
        Command::Set { key, value, expiry } => {
            db.set(&key, value, expiry);
            Frame::SimpleString("OK".into())
        }
    }
}
