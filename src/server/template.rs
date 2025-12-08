use std::path::Path;

const TEMPLATE: &str = include_str!("../../assets/template.html");

/// Render markdown content into the HTML page template.
pub fn render_page(file_path: &Path, content: &str) -> String {
    TEMPLATE
        .replace("{{file_path}}", &file_path.display().to_string())
        .replace("{{content}}", content)
}
