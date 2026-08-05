use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use axum::http::StatusCode;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use mime_guess::{
    from_path,
    mime::{APPLICATION, CHARSET, JSON, Mime, OCTET_STREAM, TEXT, XML},
};

/// Build a composite Gitignore by collecting `.gitignore` files from `base_dir` up to the
/// filesystem root. Files are added from the outermost ancestor first so that closer rules
/// take precedence, matching standard git behavior.
pub fn build_gitignore(base_dir: &Path) -> Gitignore {
    let mut paths = Vec::new();
    let mut dir = Some(base_dir);
    while let Some(d) = dir {
        let gi = d.join(".gitignore");
        if gi.exists() {
            paths.push(gi);
        }
        dir = d.parent();
    }

    if paths.is_empty() {
        return Gitignore::empty();
    }

    // Add outermost first so inner rules override
    let mut builder = GitignoreBuilder::new(base_dir);
    for path in paths.into_iter().rev() {
        let _ = builder.add(path);
    }
    builder.build().unwrap_or_else(|_| Gitignore::empty())
}

pub fn guess_content_type(path: &Path, content: &[u8]) -> String {
    let Some(mime) = from_path(path).first() else {
        return if std::str::from_utf8(content).is_ok() {
            "text/plain; charset=utf-8"
        } else {
            "application/octet-stream"
        }
        .into();
    };

    // `mime_guess` returns bare types such as `text/css`, leaving browsers to sniff the
    // encoding. Declare UTF-8 for textual types, but only when the bytes really are valid
    // UTF-8 so a legacy-encoded file is not mislabeled.
    if is_textual(&mime)
        && mime.get_param(CHARSET).is_none()
        && std::str::from_utf8(content).is_ok()
    {
        return format!("{mime}; charset=utf-8");
    }

    mime.to_string()
}

/// Whether a charset parameter is meaningful for this type. Covers `text/*`, JSON,
/// JavaScript, any `+xml` structured syntax such as `image/svg+xml`, and the `application/x-*`
/// types `mime_guess` hands back for scripts (`application/x-sh` and friends). Media families
/// such as `image/*` and `audio/*`, plus the `application/octet-stream` fallback, are excluded.
fn is_textual(mime: &Mime) -> bool {
    if mime.subtype() == JSON || mime.subtype() == XML || mime.suffix() == Some(XML) {
        return true;
    }
    (mime.type_() == TEXT || mime.type_() == APPLICATION) && mime.subtype() != OCTET_STREAM
}

pub fn relative_display(path: &Path) -> String {
    current_dir()
        .ok()
        .and_then(|cwd| path.strip_prefix(&cwd).ok().map(|p| p.display().to_string()))
        .unwrap_or_else(|| path.display().to_string())
}

pub fn resolve_safe_path(base: &Path, requested: &str) -> Result<PathBuf, StatusCode> {
    let base = base.canonicalize().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let resolved = base
        .join(requested.strip_prefix('/').unwrap_or(requested))
        .canonicalize()
        .map_err(|_| StatusCode::NOT_FOUND)?;
    if !resolved.starts_with(&base) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn textual(s: &str) -> bool {
        is_textual(&s.parse::<Mime>().unwrap())
    }

    #[test]
    fn charset_is_added_only_to_textual_types() {
        assert!(textual("text/x-python"));
        assert!(textual("text/yaml"));
        assert!(textual("application/x-sh"));
        assert!(textual("application/json"));
        assert!(textual("image/svg+xml"));

        assert!(!textual("application/octet-stream"));
        assert!(!textual("image/png"));
        assert!(!textual("font/woff2"));
    }

    #[test]
    fn charset_is_added_only_when_bytes_are_utf8() {
        let path = Path::new("a.yaml");
        assert!(guess_content_type(path, "日本語".as_bytes()).ends_with("; charset=utf-8"));
        // Shift_JIS bytes must not be labeled UTF-8.
        assert!(!guess_content_type(path, b"\x82\xa0").contains("charset"));
    }

    #[test]
    fn unknown_extension_falls_back_on_content() {
        let path = Path::new("a.unknown");
        assert_eq!(guess_content_type(path, b"hi"), "text/plain; charset=utf-8");
        assert_eq!(guess_content_type(path, b"\x82\xa0"), "application/octet-stream");
    }
}
