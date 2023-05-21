use axum::{Router, response::IntoResponse, routing::get};
use axum_sessions::extractors::{ReadableSession, WritableSession};

async fn display_handler(session: ReadableSession) -> impl IntoResponse {
    let mut count = 0;
    count = session.get("count").unwrap_or(count);
    format!(
        "Count is: {count}; visit /inc to increment and /reset to reset"
    )
}

async fn increment_handler(mut session: WritableSession) -> impl IntoResponse {
    let mut count = 1;
    count = session.get("count").map(|n: i32| n + 1).unwrap_or(count);
    session.insert("count", count).unwrap();
    format!("Count is: {count}")
}

async fn reset_handler(mut session: WritableSession) -> impl IntoResponse {
    session.destroy();
    "Count reset"
}

pub fn configure() -> Router {
    Router::new()
        .route("/", get(display_handler))
        .route("/inc", get(increment_handler))
        .route("/reset", get(reset_handler))
}