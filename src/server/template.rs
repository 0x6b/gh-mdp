use std::path::Path;

const TEMPLATE: &str = include_str!("../../assets/template.html");

/// Render markdown content into the HTML page template. `read_only` hides the
/// edit toggle, used for generated pages such as directory listings that have
/// no underlying markdown file to save back to.
pub fn render_page(file_path: &Path, content: &str, read_only: bool) -> String {
    TEMPLATE
        .replace("{{file_path}}", &file_path.display().to_string())
        .replace("{{mode_btn_attrs}}", if read_only { " hidden" } else { "" })
        .replace("{{content}}", content)
}
