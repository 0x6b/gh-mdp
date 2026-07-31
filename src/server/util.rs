use std::{
    env::current_dir,
    path::{Path, PathBuf},
};

use axum::http::StatusCode;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use mime_guess::from_path;

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
