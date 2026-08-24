use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use axum::http::StatusCode;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use mime_guess::{
    from_path,
    mime::{CHARSET, JAVASCRIPT, JSON, Mime, TEXT, XML},
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

/// Percent-encode a single path segment so it is safe inside a URL path or a
/// markdown link destination (spaces, parentheses, `#`, and non-ASCII bytes all
/// need it).
pub fn encode_segment(name: &str) -> String {
    name.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                char::from(b).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

pub fn guess_content_type(path: &Path, content: &[u8]) -> String {
    // Valid UTF-8 with no NUL byte. Deciding from the bytes keeps source files readable even
    // when the extension is unknown or mismapped, without mislabeling a legacy-encoded file.
    let is_utf8_text = !content.contains(&0) && std::str::from_utf8(content).is_ok();

    match from_path(path).first() {
        // Types where a charset is meaningful. `mime_guess` returns them bare, e.g. `text/css`,
        // which leaves the browser to sniff the encoding and mangle non-ASCII.
        Some(mime) if carries_charset(&mime) => {
            if is_utf8_text && mime.get_param(CHARSET).is_none() {
                format!("{mime}; charset=utf-8")
            } else {
                mime.to_string()
            }
        }
        // `mime_guess` is a poor source-code classifier: `.ts` is an MPEG transport stream,
        // `.java` is `application/octet-stream`, and `.go`, `.kt`, `.tsx`, `.vue` map to
        // nothing at all. Text content wins over any of those guesses.
        _ if is_utf8_text => "text/plain; charset=utf-8".into(),
        Some(mime) => mime.to_string(),
        None => "application/octet-stream".into(),
    }
}

/// Whether a `charset` parameter is meaningful for this type: `text/*`, JSON, JavaScript, and
/// any `+xml` structured syntax such as `image/svg+xml`.
fn carries_charset(mime: &Mime) -> bool {
    mime.type_() == TEXT
        || mime.subtype() == JSON
        || mime.subtype() == JAVASCRIPT
        || mime.subtype() == XML
        || mime.suffix() == Some(XML)
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

    const TEXT_BYTES: &[u8] = "日本語".as_bytes();
    // Shift_JIS "あ": valid bytes, invalid UTF-8.
    const SJIS_BYTES: &[u8] = b"\x82\xa0";
    const BINARY_BYTES: &[u8] = b"\x00\x01\x02";

    fn ct(name: &str, content: &[u8]) -> String {
        guess_content_type(Path::new(name), content)
    }

    #[test]
    fn charset_types_keep_their_type_and_gain_utf8() {
        assert!(carries_charset(&"text/css".parse().unwrap()));
        assert!(carries_charset(&"application/json".parse().unwrap()));
        assert!(carries_charset(&"image/svg+xml".parse().unwrap()));
        assert!(!carries_charset(&"image/png".parse().unwrap()));

        assert_eq!(ct("a.css", TEXT_BYTES), "text/css; charset=utf-8");
        assert_eq!(ct("a.svg", TEXT_BYTES), "image/svg+xml; charset=utf-8");
    }

    #[test]
    fn text_content_overrides_a_wrong_or_missing_guess() {
        // `.ts` guesses an MPEG stream, `.java` guesses octet-stream, `.go` guesses nothing.
        for name in ["a.ts", "a.java", "a.go", "a.sh"] {
            assert_eq!(ct(name, TEXT_BYTES), "text/plain; charset=utf-8", "{name}");
        }
    }

    #[test]
    fn non_utf8_is_never_labeled_utf8() {
        assert_eq!(ct("a.yaml", SJIS_BYTES), "text/x-yaml");
        assert_eq!(ct("a.unknown", SJIS_BYTES), "application/octet-stream");
    }

    #[test]
    fn binary_content_keeps_its_real_type() {
        assert_eq!(ct("a.png", BINARY_BYTES), "image/png");
        assert_eq!(ct("a.ts", BINARY_BYTES), "video/vnd.dlna.mpeg-tts");
        assert_eq!(ct("a.unknown", BINARY_BYTES), "application/octet-stream");
    }
}
