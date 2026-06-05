# http-path2log

A lightweight Rust HTTP server that routes log messages to files based on the URL path.

## How It Works

Send a `GET` request and the message in the URL is appended to the corresponding log file, organized by date:

```
GET /log/{level}/{msg}
```

| Level     | Log file               |
|-----------|------------------------|
| `info`    | `logs/{date}/info.log` |
| `warn`    | `logs/{date}/warn.log` |
| `error`   | `logs/{date}/error.log` |
| `debug`   | `logs/{date}/debug.log` |
| `custom`  | `logs/{date}/custom.log` |

**Log format:**

```
[2026-06-05 13:17:45.166] Your message here
```

## Quick Start

```bash
# Build
cargo build --release

# Run (default: 0.0.0.0:8080)
cargo run --release
```

## Usage Examples

```bash
# Log an info message
curl "http://localhost:8080/log/info/Server%20started"

# Log an error
curl "http://localhost:8080/log/error/Database%20connection%20failed"

# Log a warning
curl "http://localhost:8080/log/warn/Disk%20space%20low"

# Log a debug message
curl "http://localhost:8080/log/debug/User%20logged%20in"
```

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
        ├── warn.log
        ├── error.log
        ├── debug.log
        └── custom.log
```

## Dependencies

| Crate   | Purpose           |
|---------|-------------------|
| [axum](https://crates.io/crates/axum) | HTTP framework    |
| [tokio](https://crates.io/crates/tokio) | Async runtime     |
| [chrono](https://crates.io/crates/chrono) | Timestamps        |

## License

This project is licensed under the [MIT License](LICENSE).
