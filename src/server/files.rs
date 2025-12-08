use std::sync::Arc;

use axum::{
    extract::{self, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::{Html, IntoResponse},
};
use tokio::fs::read;

use super::{markdown::render, state::AppState, template::render_page, util};

pub async fn serve_file(
    State(state): State<Arc<AppState>>,
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    let Some(base_dir) = state.file_path.parent() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let resolved = match util::resolve_safe_path(base_dir, &path) {
        Ok(p) => p,
        Err(status) => return status.into_response(),
    };

    // Render markdown files with template
    if resolved.extension().is_some_and(|ext| ext == "md") {
        let content = render(&resolved);
        return Html(render_page(&resolved, &content)).into_response();
    }

    // Read and serve static file
    let Ok(content) = read(&resolved).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let content_type = util::guess_content_type(&resolved, &content);
    ([(CONTENT_TYPE, content_type)], content).into_response()
}
