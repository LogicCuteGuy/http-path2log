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
    sync::Arc,
};

#[derive(Clone)]
struct AppState {
    log_dir: PathBuf,
}

#[tokio::main]
async fn main() {
    let log_dir = PathBuf::from("logs");
    fs::create_dir_all(&log_dir).expect("Failed to create log directory");

    let state = AppState {
        log_dir,
    };

    let app = Router::new()
        .route("/log/{level}/{msg}", get(handle_log))
        .route("/", get(index))
        .with_state(state);

    let addr = "0.0.0.0:8080";
    println!("🚀 Server running on http://{addr}");
    println!("   GET /log/{{info,warn,error,debug,custom}}/{{msg}}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> &'static str {
    "http-path2log server\n\nUsage: GET /log/{level}/{msg}\nLevels: info, warn, error, debug, custom\n"
}

async fn handle_log(
    Path((level, msg)): Path<(String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let valid_levels = ["info", "warn", "error", "debug", "custom"];
    let level = level.to_lowercase();

    if !valid_levels.contains(&level.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            format!("Invalid level '{}'. Use: info, warn, error, debug, custom", level),
        );
    }

    let today = Local::now().format("%Y-%m-%d").to_string();
    let log_dir = state.log_dir.join(&today);
    fs::create_dir_all(&log_dir).expect("Failed to create date directory");

    let log_file = log_dir.join(format!("{}.log", level));
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
