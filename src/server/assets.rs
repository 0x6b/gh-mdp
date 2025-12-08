use std::sync::Arc;

use axum::{
    extract::{self, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::IntoResponse,
};
use tokio::fs::read;

use super::{state::AppState, util};

const FAVICON_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 208 128"><rect width="198" height="118" x="5" y="5" ry="10" stroke="#000" stroke-width="10" fill="none"/><path d="M30 98V30h20l20 25 20-25h20v68H90V59L70 84 50 59v39zm125 0l-30-33h20V30h20v35h20z"/></svg>"##;

const GITHUB_MARKDOWN_CSS: &[u8] = include_bytes!("../../assets/github-markdown.min.css");
const HIGHLIGHT_CSS: &[u8] = include_bytes!("../../assets/highlight-github.min.css");
const HIGHLIGHT_DARK_CSS: &[u8] = include_bytes!("../../assets/highlight-github-dark.min.css");
const HIGHLIGHT_JS: &[u8] = include_bytes!("../../assets/highlight.min.js");
const MORPHDOM_JS: &[u8] = include_bytes!("../../assets/morphdom.min.js");
const MERMAID_JS: &[u8] = include_bytes!("../../assets/mermaid.min.js");

pub async fn serve_favicon() -> impl IntoResponse {
    ([(CONTENT_TYPE, "image/svg+xml")], FAVICON_SVG)
}

pub async fn serve_asset(
    State(state): State<Arc<AppState>>,
    extract::Path(path): extract::Path<String>,
) -> impl IntoResponse {
    // Check embedded assets first
    let embedded: Option<(&str, &[u8])> = match path.as_str() {
        "github-markdown.min.css" => Some(("text/css", GITHUB_MARKDOWN_CSS)),
        "highlight-github.min.css" => Some(("text/css", HIGHLIGHT_CSS)),
        "highlight-github-dark.min.css" => Some(("text/css", HIGHLIGHT_DARK_CSS)),
        "highlight.min.js" => Some(("text/javascript", HIGHLIGHT_JS)),
        "morphdom.min.js" => Some(("text/javascript", MORPHDOM_JS)),
        "mermaid.min.js" => Some(("text/javascript", MERMAID_JS)),
        _ => None,
    };

    if let Some((content_type, body)) = embedded {
        return ([(CONTENT_TYPE, content_type)], body).into_response();
    }

    // Fall back to filesystem for user assets in /assets directory
    let Some(base_dir) = state.file_path.parent() else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    let assets_path = format!("assets/{path}");
    let resolved = match util::resolve_safe_path(base_dir, &assets_path) {
        Ok(p) => p,
        Err(status) => return status.into_response(),
    };

    let Ok(content) = read(&resolved).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let content_type = util::guess_content_type(&resolved, &content);
    ([(CONTENT_TYPE, content_type)], content).into_response()
}
