# 📦 padis

A Redis Clone written in Rust to explore system design and multi-threading.

## Features

- RESP protocol parsing
- TCP connection handling with async I/O
- Commands: `PING`, `ECHO`, `GET`, `SET` (with expiry)
- Thread-safe in-memory key-value store
- Key expiration support
- Unit and integration testing
- CI with formatting, linting and testing

## Usage

Start the server:
```bash
cargo run
```

Connect with redis-cli:
```bash
redis-cli -p 6379
> PING
PONG
> SET foo bar
OK
> GET foo
"bar"
> SET temp value PX 5000
OK
> GET temp
"value"
# ... wait 5 seconds
> GET temp
(nil)
```

## Project Structure
```
src/
├── lib.rs         # Public exports
├── frame.rs       # RESP protocol parser
├── connection.rs  # Async TCP connection handling
├── cmd.rs         # Command parsing
├── db.rs          # Thread-safe key-value store
└── server.rs      # Server loop and request handling
```

## Running Tests
```bash
cargo test
```

## What I Learned

Built this to learn Rust fundamentals:

- Ownership, borrowing, and lifetimes
- Async/await with Tokio
- Error handling patterns
- Concurrency with `Arc<Mutex<T>>`
- The `bytes` crate for buffer management

See my [writeups](https://github.com/PaddyConnolly/writing.git) for detailed notes.

## Acknowledgments

Inspired by [mini-redis](https://github.com/tokio-rs/mini-redis) from the Tokio project
