use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{StatusCode, header::CONTENT_TYPE},
    response::{Html, IntoResponse},
};
use tokio::fs::{read, read_to_string};

use super::{
    listing::render_listing,
    markdown::render,
    state::AppState,
    template::render_page,
    util::{guess_content_type, relative_display, resolve_safe_path},
};

pub async fn serve_file(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let resolved = match resolve_safe_path(&state.base_dir, &path) {
        Ok(p) => p,
        Err(s) => return s.into_response(),
    };

    // Directories get the same generated listing as a directory root, minus the
    // edit toggle since there is no file to save back to.
    if resolved.is_dir() {
        let markdown = render_listing(&resolved, &state.base_dir);
        let file = relative_display(&resolved);
        let url = state.file_url(&resolved);
        let html = render(&markdown, &file, &url);
        return Html(render_page(&resolved, &html, true)).into_response();
    }

    if resolved.extension().is_some_and(|ext| ext == "md") {
        let Ok(markdown) = read_to_string(&resolved).await else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let file = relative_display(&resolved);
        let url = state.file_url(&resolved);
        return Html(render_page(&resolved, &render(&markdown, &file, &url), false))
            .into_response();
    }

    let Ok(content) = read(&resolved).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    ([(CONTENT_TYPE, guess_content_type(&resolved, &content))], content).into_response()
}
