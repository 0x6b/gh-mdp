use std::path;

use axum::{
    extract,
    http::{StatusCode, header::CONTENT_TYPE},
    response::IntoResponse,
};

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

pub async fn handler(extract::Path(path): extract::Path<String>) -> impl IntoResponse {
    let (content_type, body): (&str, &[u8]) = match path.as_str() {
        "github-markdown.min.css" => ("text/css", GITHUB_MARKDOWN_CSS),
        "highlight-github.min.css" => ("text/css", HIGHLIGHT_CSS),
        "highlight-github-dark.min.css" => ("text/css", HIGHLIGHT_DARK_CSS),
        "highlight.min.js" => ("text/javascript", HIGHLIGHT_JS),
        "morphdom.min.js" => ("text/javascript", MORPHDOM_JS),
        "mermaid.min.js" => ("text/javascript", MERMAID_JS),
        _ => return (StatusCode::NOT_FOUND, "Not found").into_response(),
    };
    ([(CONTENT_TYPE, content_type)], body).into_response()
}
