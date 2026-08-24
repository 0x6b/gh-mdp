use std::{fmt::Write, path::Path};

use super::util::{encode_segment, escape_html};

const TEMPLATE: &str = include_str!("../../assets/template.html");

/// Render markdown content into the HTML page template. `read_only` hides the
/// edit toggle, used for generated pages such as directory listings that have
/// no underlying markdown file to save back to.
pub fn render_page(file_path: &Path, base_dir: &Path, content: &str, read_only: bool) -> String {
    TEMPLATE
        .replace("{{file_path}}", &escape_html(&file_path.display().to_string()))
        .replace("{{breadcrumb}}", &breadcrumb(file_path, base_dir))
        .replace("{{mode_btn_attrs}}", if read_only { " hidden" } else { "" })
        .replace("{{content}}", content)
}

/// The header path, with each directory at or below `base_dir` linked to its
/// listing. Nothing above `base_dir` is served, so that part stays inside the
/// first crumb instead of becoming a dead link. The last segment is the page
/// itself and is left as text.
fn breadcrumb(file_path: &Path, base_dir: &Path) -> String {
    let mut html = format!("<a href=\"/\">{}</a>", escape_html(&base_dir.display().to_string()));
    let Ok(rel) = file_path.strip_prefix(base_dir) else {
        return html;
    };

    let last = rel.components().count().saturating_sub(1);
    let mut url = String::new();
    for (i, component) in rel.components().enumerate() {
        let name = component.as_os_str().to_string_lossy();
        let _ = write!(url, "/{}", encode_segment(&name));
        let name = escape_html(&name);
        html.push_str("<span class=\"header-sep\">/</span>");
        if i == last {
            html.push_str(&name);
        } else {
            let _ = write!(html, "<a href=\"{url}/\">{name}</a>");
        }
    }
    html
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crumb(base: &str, file: &str) -> String {
        breadcrumb(Path::new(file), Path::new(base))
    }

    #[test]
    fn the_previewed_root_is_one_crumb_pointing_at_the_listing() {
        assert_eq!(crumb("/w", "/w"), r#"<a href="/">/w</a>"#);
    }

    #[test]
    fn directories_below_the_root_link_to_their_own_listing() {
        assert_eq!(
            crumb("/w", "/w/docs/sample.html"),
            r#"<a href="/">/w</a><span class="header-sep">/</span><a href="/docs/">docs</a><span class="header-sep">/</span>sample.html"#
        );
    }

    #[test]
    fn names_are_escaped_and_percent_encoded() {
        assert_eq!(
            crumb("/w", "/w/a&b/c d.md"),
            r#"<a href="/">/w</a><span class="header-sep">/</span><a href="/a%26b/">a&amp;b</a><span class="header-sep">/</span>c d.md"#
        );
    }
}
