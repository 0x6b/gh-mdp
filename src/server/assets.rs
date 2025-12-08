use std::{path, sync::Arc};

use axum::{
    extract::{self, State},
    http::{StatusCode, header::CONTENT_TYPE},
    response::IntoResponse,
};

use super::state::AppState;

const FAVICON_SVG: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 208 128"><rect width="198" height="118" x="5" y="5" ry="10" stroke="#000" stroke-width="10" fill="none"/><path d="M30 98V30h20l20 25 20-25h20v68H90V59L70 84 50 59v39zm125 0l-30-33h20V30h20v35h20z"/></svg>"##;

const TEMPLATE: &str = include_str!("../../assets/template.html");
const GITHUB_MARKDOWN_CSS: &[u8] = include_bytes!("../../assets/github-markdown.min.css");
const HIGHLIGHT_CSS: &[u8] = include_bytes!("../../assets/highlight-github.min.css");
const HIGHLIGHT_DARK_CSS: &[u8] = include_bytes!("../../assets/highlight-github-dark.min.css");
const HIGHLIGHT_JS: &[u8] = include_bytes!("../../assets/highlight.min.js");
const MORPHDOM_JS: &[u8] = include_bytes!("../../assets/morphdom.min.js");
const MERMAID_JS: &[u8] = include_bytes!("../../assets/mermaid.min.js");

pub fn render_page(file_path: &path::Path, content: &str) -> String {
    TEMPLATE
        .replace("{{file_path}}", &file_path.display().to_string())
        .replace("{{content}}", content)
}

pub async fn favicon_handler() -> impl IntoResponse {
    ([(CONTENT_TYPE, "image/svg+xml")], FAVICON_SVG)
}

pub async fn handler(
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

    let requested = base_dir.join("assets").join(&path);

    let Ok(resolved) = requested.canonicalize() else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if !resolved.starts_with(base_dir) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let Ok(content) = read(&resolved).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let content_type = from_path(&resolved)
        .first()
        .map(|m| m.to_string())
        .unwrap_or_else(|| {
            if std::str::from_utf8(&content).is_ok() {
                "text/plain; charset=utf-8".to_string()
            } else {
                "application/octet-stream".to_string()
            }
        });

    ([(CONTENT_TYPE, content_type)], content).into_response()
}
