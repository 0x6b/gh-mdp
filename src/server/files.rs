use std::sync::Arc;

use axum::{
    extract::{self, State},
    http::{StatusCode, header::CONTENT_TYPE},
    response::{Html, IntoResponse},
};
use extract::Path;
use mime_guess::from_path;
use tokio::fs::read;

use super::{assets::render_page, markdown::render, state::AppState};

pub async fn handler(
    State(state): State<Arc<AppState>>,
    extract::Path(path): Path<String>,
) -> impl IntoResponse {
    // Get base directory from markdown file path
    let Some(base_dir) = state.file_path.parent() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    // Reject absolute paths
    if path.starts_with('/') {
        return StatusCode::BAD_REQUEST.into_response();
    }

    // Resolve path relative to base directory
    let requested = base_dir.join(&path);

    // Security: canonicalize and verify path stays within base directory
    let Ok(resolved) = requested.canonicalize() else {
        return StatusCode::NOT_FOUND.into_response();
    };

    // base_dir is already canonical since file_path is canonicalized in main.rs
    if !resolved.starts_with(base_dir) {
        return StatusCode::FORBIDDEN.into_response();
    }

    // Render markdown files with template
    if resolved.extension().is_some_and(|ext| ext == "md") {
        let content = render(&resolved);
        return Html(render_page(&resolved, &content)).into_response();
    }

    // Read file
    let Ok(content) = read(&resolved).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    // Determine Content-Type from file extension
    let content_type = from_path(&resolved)
        .first()
        .map(|m| m.to_string())
        .unwrap_or_else(|| {
            // No extension or unknown extension: check if valid UTF-8 text
            if std::str::from_utf8(&content).is_ok() {
                "text/plain; charset=utf-8".to_string()
            } else {
                "application/octet-stream".to_string()
            }
        });

    ([(CONTENT_TYPE, content_type)], content).into_response()
}
