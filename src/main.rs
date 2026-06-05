use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::Local;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

#[derive(Clone)]
struct AppState {
    log_dir: PathBuf,
}

fn parse_port() -> u16 {
    // --port <N> takes priority
    let args: Vec<String> = std::env::args().collect();
    if let Some(pos) = args.iter().position(|a| a == "--port") {
        if let Some(val) = args.get(pos + 1) {
            if let Ok(p) = val.parse::<u16>() {
                return p;
            }
            eprintln!("Invalid port: {val}");
            std::process::exit(1);
        }
        eprintln!("--port requires a number");
        std::process::exit(1);
    }

    // Then PORT env var
    if let Ok(val) = std::env::var("PORT") {
        if let Ok(p) = val.parse::<u16>() {
            return p;
        }
        eprintln!("Invalid PORT env: {val}");
        std::process::exit(1);
    }

    8080
}

#[tokio::main]
async fn main() {
    let port = parse_port();
    let log_dir = PathBuf::from("logs");
    fs::create_dir_all(&log_dir).expect("Failed to create log directory");

    let state = AppState {
        log_dir,
    };

    let app = Router::new()
        .route("/log/{*path}", get(handle_log))
        .route("/", get(index))
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    println!("🚀 Server running on http://{addr}");
    println!("   Port: {port} (override with --port <N> or PORT env var)");
    println!("   GET /log/{{topic/...}}/{{msg}}");
    println!("   GET /log/{{topic/...}}?msg={{msg}}");
    println!("   Topics: any path (e.g. info, app/auth/deploy)");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> &'static str {
    "http-path2log server\n\nUsage:\n  GET /log/{topic}/{msg}\n  GET /log/{path/...}/{msg}\n\nExamples:\n  /log/info/Hello World\n  /log/app/auth/login-failed/User not found\n  /log/deploy/prod/backend/restart\n\nOutput: logs/{date}/{topic}.log\n\nPort:\n  --port <N>  flag\n  PORT=<N>    env var\n  default: 8080\n"
}

async fn handle_log(
    Path(path): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Split into segments
    let mut segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.len() < 2 {
        return (
            StatusCode::BAD_REQUEST,
            "Need at least a topic and message: /log/{topic}/{msg}".to_string(),
        );
    }

    // Last segment is the message, everything before is directory structure
    let msg = segments.pop().unwrap(); // message
    let filename: String = segments
        .last()
        .unwrap()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase();

    if filename.is_empty() || msg.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "Both topic and message are required. Use /log/{topic/...}/{msg}".to_string(),
        );
    }

    let today = Local::now().format("%Y-%m-%d").to_string();
    let log_dir = state.log_dir.join(&today);

    // Build subdirectory path from all segments except the last two
    // (last = message, second-to-last = filename, rest = dirs)
    let dir_segments = &segments[..segments.len() - 1];
    let mut full_dir = log_dir.clone();
    for seg in dir_segments {
        let safe: String = seg
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>()
            .to_lowercase();
        full_dir = full_dir.join(safe);
    }
    fs::create_dir_all(&full_dir).expect("Failed to create log directory");

    let log_file = full_dir.join(format!("{}.log", filename));
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");

    let entry = format!("[{}] {}\n", timestamp, msg);

    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
    {
        Ok(mut file) => {
            if let Err(e) = file.write_all(entry.as_bytes()) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to write log: {e}"),
                );
            }
            (
                StatusCode::OK,
                format!("✓ Logged to {} — {}", log_file.display(), msg),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to open log file: {e}"),
        ),
    }
}
