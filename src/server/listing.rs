use std::{
    fmt::Write,
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
};

use super::util::{build_gitignore, encode_segment, relative_display};

/// The files a directory is represented by, in the order they are looked for.
const DEFAULT_FILES: [&str; 2] = ["index.md", "README.md"];

/// The markdown file that stands for `dir`, if it has one.
pub fn default_markdown(dir: &Path) -> Option<PathBuf> {
    DEFAULT_FILES
        .into_iter()
        .map(|name| dir.join(name))
        .find(|path| path.is_file())
}

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
///
/// A directory that has an `index.md` or `README.md` shows it below the file
/// list, the way the directory itself would be read.
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

    if dir == base_dir && dirs.is_empty() && files.is_empty() {
        markdown.push_str("_Empty directory._\n");
        return markdown;
    }

    // Wrapped so the stylesheet can set the entries in a monospace face without
    // reaching the lists of the markdown file rendered below them.
    markdown.push_str("<div class=\"file-list\">\n\n");
    if dir != base_dir {
        markdown.push_str("- [../](../)\n");
    }

    for name in &dirs {
        let _ = writeln!(markdown, "- [{}/]({}/)", escape_markdown(name), encode_segment(name));
    }
    for name in &files {
        let _ = writeln!(markdown, "- [{}]({})", escape_markdown(name), encode_segment(name));
    }
    markdown.push_str("\n</div>\n");

    if let Some(path) = default_markdown(dir)
        && let Ok(content) = read_to_string(path)
    {
        let _ = write!(markdown, "\n---\n\n{content}");
    }

    markdown
}

#[cfg(test)]
mod tests {
    use std::{
        env::temp_dir,
        fs::{create_dir_all, write},
    };

    use super::*;

    fn fixture(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = temp_dir().join(format!("gh-mdp-listing-{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        create_dir_all(&dir).unwrap();
        for (name, content) in files {
            write(dir.join(name), content).unwrap();
        }
        dir
    }

    #[test]
    fn a_directory_shows_its_readme_below_the_file_list() {
        let dir = fixture("readme", &[("README.md", "# Hello\n"), ("other.txt", "")]);
        let markdown = render_listing(&dir, &dir);
        assert!(markdown.contains("<div class=\"file-list\">"), "{markdown}");
        assert!(markdown.contains("- [README\\.md](README.md)"), "{markdown}");
        assert!(markdown.ends_with("\n---\n\n# Hello\n"), "{markdown}");
    }

    #[test]
    fn index_wins_over_readme() {
        let dir = fixture("index", &[("README.md", "# Readme\n"), ("index.md", "# Index\n")]);
        assert!(render_listing(&dir, &dir).ends_with("# Index\n"));
    }

    #[test]
    fn a_directory_without_one_is_just_the_file_list() {
        let dir = fixture("plain", &[("notes.md", "# Notes\n")]);
        let markdown = render_listing(&dir, &dir);
        assert!(!markdown.contains("---"), "{markdown}");
        assert!(markdown.contains("- [notes\\.md](notes.md)\n"), "{markdown}");
        assert!(markdown.ends_with("\n</div>\n"), "{markdown}");
    }
}
