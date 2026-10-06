use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{
        StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    response::IntoResponse,
};
use tokio::fs::read;

use super::{
    state::AppState,
    util::{guess_content_type, resolve_safe_path},
};

const FAVICON: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><path d="M14.85 3c.63 0 1.15.52 1.14 1.15v7.7c0 .63-.51 1.15-1.15 1.15H1.15C.52 13 0 12.48 0 11.84V4.15C0 3.52.52 3 1.15 3ZM9 11V5H7L5.5 7 4 5H2v6h2V8l1.5 1.92L7 8v3Zm2.99.5L14.5 8H13V5h-2v3H9.5Z"/></svg>"##;

const EMBEDDED: &[(&str, &str, &[u8])] = &[
    (
        "github-markdown.min.css",
        "text/css; charset=utf-8",
        include_bytes!("../../assets/github-markdown.min.css"),
    ),
    (
        "highlight-github.min.css",
        "text/css; charset=utf-8",
        include_bytes!("../../assets/highlight-github.min.css"),
    ),
    (
        "highlight-github-dark.min.css",
        "text/css; charset=utf-8",
        include_bytes!("../../assets/highlight-github-dark.min.css"),
    ),
    (
        "highlight.min.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../../assets/highlight.min.js"),
    ),
    (
        "morphdom.min.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../../assets/morphdom.min.js"),
    ),
    (
        "mermaid.min.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../../assets/mermaid.min.js"),
    ),
    (
        "overtype.min.js",
        "text/javascript; charset=utf-8",
        include_bytes!("../../assets/overtype.min.js"),
    ),
];

pub async fn serve_favicon() -> impl IntoResponse {
    ([(CONTENT_TYPE, "image/svg+xml; charset=utf-8")], FAVICON)
}

pub async fn serve_asset(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> impl IntoResponse {
    if let Some((_, ct, body)) = EMBEDDED.iter().find(|(name, _, _)| *name == path) {
        return (
            [(CONTENT_TYPE, *ct), (CACHE_CONTROL, "public, max-age=31536000, immutable")],
            *body,
        )
            .into_response();
    }

    let Some(base_dir) = state.file_path.parent() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let resolved = match resolve_safe_path(base_dir, &format!("assets/{path}")) {
        Ok(p) => p,
        Err(s) => return s.into_response(),
    };

    let Ok(content) = read(&resolved).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    ([(CONTENT_TYPE, guess_content_type(&resolved, &content))], content).into_response()
}
