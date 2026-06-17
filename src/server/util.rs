use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use axum::http::StatusCode;
use mime_guess::from_path;

pub fn guess_content_type(path: &Path, content: &[u8]) -> String {
    from_path(path).first().map_or_else(
        || {
            if std::str::from_utf8(content).is_ok() {
                "text/plain; charset=utf-8"
            } else {
                "application/octet-stream"
            }
            .into()
        },
        |m| m.to_string(),
    )
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
