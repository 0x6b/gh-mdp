use std::path;

use axum::{
    extract,
    http::{StatusCode, header::CONTENT_TYPE},
    response::IntoResponse,
};

const TEMPLATE: &str = include_str!("../../assets/template.html");
const GITHUB_MARKDOWN_CSS: &[u8] = include_bytes!("../../assets/github-markdown.min.css");
const HIGHLIGHT_CSS: &[u8] = include_bytes!("../../assets/highlight-github.min.css");
const HIGHLIGHT_DARK_CSS: &[u8] = include_bytes!("../../assets/highlight-github-dark.min.css");
const HIGHLIGHT_JS: &[u8] = include_bytes!("../../assets/highlight.min.js");
const MORPHDOM_JS: &[u8] = include_bytes!("../../assets/morphdom.min.js");

pub fn render_page(file_path: &path::Path, content: &str) -> String {
    TEMPLATE
        .replace("{{file_path}}", &file_path.display().to_string())
        .replace("{{content}}", content)
}

pub async fn handler(extract::Path(path): extract::Path<String>) -> impl IntoResponse {
    let (content_type, body): (&str, &[u8]) = match path.as_str() {
        "github-markdown.min.css" => ("text/css", GITHUB_MARKDOWN_CSS),
        "highlight-github.min.css" => ("text/css", HIGHLIGHT_CSS),
        "highlight-github-dark.min.css" => ("text/css", HIGHLIGHT_DARK_CSS),
        "highlight.min.js" => ("text/javascript", HIGHLIGHT_JS),
        "morphdom.min.js" => ("text/javascript", MORPHDOM_JS),
        _ => return (StatusCode::NOT_FOUND, "Not found").into_response(),
    };
    ([(CONTENT_TYPE, content_type)], body).into_response()
}
