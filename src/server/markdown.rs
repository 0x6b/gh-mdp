use std::{fs::read_to_string, path::Path, time::Instant};

use pulldown_cmark::{Options, Parser, html::push_html};
use tracing::info;

const OPTIONS: Options = Options::ENABLE_GFM
    .union(Options::ENABLE_TABLES)
    .union(Options::ENABLE_FOOTNOTES)
    .union(Options::ENABLE_STRIKETHROUGH)
    .union(Options::ENABLE_TASKLISTS)
    .union(Options::ENABLE_SMART_PUNCTUATION);

pub fn render(path: &Path) -> String {
    let start = Instant::now();
    let content = read_to_string(path).unwrap_or_else(|e| format!("Error reading file: {e}"));
    let parser = Parser::new_ext(&content, OPTIONS);
    let mut html_output = String::new();
    push_html(&mut html_output, parser);
    info!(latency = ?start.elapsed(), "Markdown rendered");
    html_output
}
