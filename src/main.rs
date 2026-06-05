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
    println!("   GET /log/{{topic}}/{{msg}}");
    println!("   Topics: any name (e.g. info, error, MyProject, deploy-status)");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn index() -> &'static str {
    "http-path2log server\n\nUsage: GET /log/{topic}/{msg}\n\nAny topic name is valid — it becomes the log filename.\nExamples:\n  /log/info/Hello World\n  /log/deploy/Production deploy succeeded\n  /log/MyProject/Something happened\n\nFiles are created under logs/{date}/{topic}.log\n"
}

async fn handle_log(
    Path((level, msg)): Path<(String, String)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let level: String = level
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase();

    if level.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "Level/topic name cannot be empty".to_string(),
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
