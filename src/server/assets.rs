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

const FAVICON: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 208 128"><rect width="198" height="118" x="5" y="5" ry="10" stroke="#000" stroke-width="10" fill="none"/><path d="M30 98V30h20l20 25 20-25h20v68H90V59L70 84 50 59v39zm125 0l-30-33h20V30h20v35h20z"/></svg>"##;

const EMBEDDED: &[(&str, &str, &[u8])] = &[
    ("github-markdown.min.css", "text/css", include_bytes!("../../assets/github-markdown.min.css")),
    (
        "highlight-github.min.css",
        "text/css",
        include_bytes!("../../assets/highlight-github.min.css"),
    ),
    (
        "highlight-github-dark.min.css",
        "text/css",
        include_bytes!("../../assets/highlight-github-dark.min.css"),
    ),
    ("highlight.min.js", "text/javascript", include_bytes!("../../assets/highlight.min.js")),
    ("morphdom.min.js", "text/javascript", include_bytes!("../../assets/morphdom.min.js")),
    ("mermaid.min.js", "text/javascript", include_bytes!("../../assets/mermaid.min.js")),
    ("overtype.min.js", "text/javascript", include_bytes!("../../assets/overtype.min.js")),
];

pub async fn serve_favicon() -> impl IntoResponse {
    ([(CONTENT_TYPE, "image/svg+xml")], FAVICON)
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
