use std::path::{Path, PathBuf};

use axum::http::StatusCode;
use mime_guess::from_path;

/// Guess the Content-Type for a file based on its extension.
/// Falls back to UTF-8 text detection for files without extensions.
pub fn guess_content_type(path: &Path, content: &[u8]) -> String {
    from_path(path)
        .first()
        .map(|m| m.to_string())
        .unwrap_or_else(|| {
            if std::str::from_utf8(content).is_ok() {
                "text/plain; charset=utf-8".to_string()
            } else {
                "application/octet-stream".to_string()
            }
        })
}

/// Resolve a requested path relative to base directory with security validation.
/// Returns error status codes for invalid or unauthorized paths.
pub fn resolve_safe_path(base_dir: &Path, requested: &str) -> Result<PathBuf, StatusCode> {
    if requested.starts_with('/') {
        return Err(StatusCode::BAD_REQUEST);
    }

    let requested_path = base_dir.join(requested);

    let resolved = requested_path
        .canonicalize()
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if !resolved.starts_with(base_dir) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(resolved)
}
