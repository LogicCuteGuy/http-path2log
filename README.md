# http-path2log

A lightweight Rust HTTP server that routes log messages to files based on the URL path.

## How It Works

Send a `GET` request — the URL path determines the log file, and the last segment becomes the message:

```
GET /log/{topic/...}/{msg}
```

Any topic name is valid. Nested paths create subdirectories automatically.

| Request | Log file |
|---------|----------|
| `/log/info/Hello` | `logs/{date}/info.log` |
| `/log/error/DB down` | `logs/{date}/error.log` |
| `/log/app/auth/login-failed` | `logs/{date}/app.log` |
| `/log/deploy/prod/backend/restart` | `logs/{date}/deploy/prod/backend.log` |

**Log format:**

```
[2026-06-05 13:17:45.166] Hello
```

## Quick Start

```bash
# Build
cargo build --release

# Run on default port (8080)
cargo run --release

# Run on a custom port (flag takes priority)
cargo run --release -- --port 3000

# Or via environment variable
PORT=9090 cargo run --release
```

## Usage Examples

```bash
# Simple flat topic
curl "http://localhost:8080/log/info/Server%20started"
curl "http://localhost:8080/log/error/Database%20connection%20failed"

# Nested topics
curl "http://localhost:8080/log/app/auth/login-failed/User%20not%20found"
curl "http://localhost:8080/log/deploy/prod/backend/restart"

# Custom topics
curl "http://localhost:8080/log/monitoring/CPU%20at%2099%25"
curl "http://localhost:8080/log/MyProject/Build%20succeeded"
```

## Port Configuration

The server port can be set in three ways (in order of priority):

| Priority | Method | Example |
|----------|--------|---------|
| 1 | `--port` argument | `cargo run -- --port 3000` |
| 2 | `PORT` env var | `PORT=9090 cargo run` |
| 3 | Default | `8080` |

## Project Structure

```
http-path2log/
├── Cargo.toml
├── LICENSE
├── README.md
├── src/
│   └── main.rs
└── logs/           # created at runtime
    └── YYYY-MM-DD/
        ├── info.log
        ├── error.log
        ├── app/
        │   └── auth.log
        └── deploy/
            └── prod/
                └── backend.log
```

## URL Routing

The path after `/log/` maps directly to the filesystem:

- **Last segment** → message content (written to the log file)
- **Second-to-last segment** → log filename (`.log` appended)
- **All preceding segments** → subdirectory structure

Examples:

| URL Path | File Created | Message |
|----------|-------------|---------|
| `/log/info/Hello` | `logs/{date}/info.log` | `Hello` |
| `/log/test/file/World` | `logs/{date}/test/file.log` | `World` |
| `/log/a/b/c/msg` | `logs/{date}/a/b/c.log` | `msg` |

## Dependencies

| Crate | Purpose |
|-------|---------|
| [axum](https://crates.io/crates/axum) | HTTP framework |
| [tokio](https://crates.io/crates/tokio) | Async runtime |
| [chrono](https://crates.io/crates/chrono) | Timestamps |

## License

This project is licensed under the [MIT License](LICENSE).
