use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{StatusCode, header::CONTENT_TYPE},
    response::{Html, IntoResponse},
};
use tokio::fs::read;

use super::{
    markdown::render,
    state::AppState,
    template::render_page,
    util::{guess_content_type, resolve_safe_path},
};

pub async fn serve_file(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let Some(base_dir) = state.file_path.parent() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let resolved = match resolve_safe_path(base_dir, &path) {
        Ok(p) => p,
        Err(s) => return s.into_response(),
    };

    if resolved.extension().is_some_and(|ext| ext == "md") {
        return Html(render_page(&resolved, &render(&resolved))).into_response();
    }

    let Ok(content) = read(&resolved).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    ([(CONTENT_TYPE, guess_content_type(&resolved, &content))], content).into_response()
}
