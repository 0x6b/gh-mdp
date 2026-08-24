use std::{fmt::Write, fs::read_dir, path::Path};

use super::util::{build_gitignore, encode_segment, relative_display};

/// Backslash-escape ASCII punctuation that would otherwise be markdown syntax.
fn escape_markdown(text: &str) -> String {
    text.chars().fold(String::with_capacity(text.len()), |mut out, c| {
        if c.is_ascii_punctuation() {
            out.push('\\');
        }
        out.push(c);
        out
    })
}

/// Build a markdown directory listing for `dir`, used as the preview when a
/// directory has no `index.md` or `README.md`. Dotfiles and gitignored entries
/// are skipped; directories are listed before files. A parent link is added
/// unless `dir` is `base_dir`, which nothing is served above.
///
/// Directory links keep a trailing slash so that relative links on the rendered
/// page resolve against the directory itself.
pub fn render_listing(dir: &Path, base_dir: &Path) -> String {
    let gitignore = build_gitignore(dir);
    let (mut dirs, mut files) = (Vec::new(), Vec::new());

    if let Ok(entries) = read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let is_dir = entry.file_type().is_ok_and(|t| t.is_dir());
            if gitignore
                .matched_path_or_any_parents(entry.path(), is_dir)
                .is_ignore()
            {
                continue;
            }
            if is_dir { dirs.push(name) } else { files.push(name) }
        }
    }

    dirs.sort_unstable();
    files.sort_unstable();

    // `relative_display` yields "" for the current directory itself; fall back to
    // its name so the heading says something useful.
    let title = match relative_display(dir) {
        s if s.is_empty() => dir
            .file_name()
            .unwrap_or(dir.as_os_str())
            .to_string_lossy()
            .into_owned(),
        s => s,
    };
    let mut markdown = format!("# {}\n\n", escape_markdown(&title));

    if dir != base_dir {
        markdown.push_str("- [../](../)\n");
    } else if dirs.is_empty() && files.is_empty() {
        markdown.push_str("_Empty directory._\n");
        return markdown;
    }

    for name in &dirs {
        let _ = writeln!(markdown, "- [{}/]({}/)", escape_markdown(name), encode_segment(name));
    }
    for name in &files {
        let _ = writeln!(markdown, "- [{}]({})", escape_markdown(name), encode_segment(name));
    }

    markdown
}
