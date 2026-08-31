use std::{path::Path as FsPath, sync::Arc};

use axum::{
    extract::{OriginalUri, Path, State},
    http::{StatusCode, Uri, header::CONTENT_TYPE},
    response::{Html, IntoResponse, Redirect},
};
use tokio::fs::{read, read_to_string};

use super::{
    listing::render_listing,
    markdown::render,
    state::AppState,
    template::render_page,
    util::{guess_content_type, relative_display, resolve_safe_path},
};

pub async fn serve_file(
    State(state): State<Arc<AppState>>,
    OriginalUri(uri): OriginalUri,
    Path(path): Path<String>,
) -> impl IntoResponse {
    let resolved = match resolve_safe_path(&state.base_dir, &path) {
        Ok(p) => p,
        Err(s) => return s.into_response(),
    };

    // Directories get the same generated listing as a directory root, minus the
    // edit toggle since there is no file to save back to.
    if resolved.is_dir() {
        // A relative link on /docs resolves from /, while the same link on
        // /docs/ resolves from /docs/. Normalize directory URLs so entries
        // in generated listings retain their directory prefix.
        if let Some(location) = directory_redirect(&uri) {
            return Redirect::permanent(&location).into_response();
        }
        return Html(render_directory(&state, &resolved)).into_response();
    }

    if resolved.extension().is_some_and(|ext| ext == "md") {
        let Ok(markdown) = read_to_string(&resolved).await else {
            return StatusCode::NOT_FOUND.into_response();
        };
        let file = relative_display(&resolved);
        let url = state.file_url(&resolved);
        return Html(render_page(
            &resolved,
            &state.base_dir,
            &render(&markdown, &file, &url),
            false,
        ))
        .into_response();
    }

    let Ok(content) = read(&resolved).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    ([(CONTENT_TYPE, guess_content_type(&resolved, &content))], content).into_response()
}

/// Render a directory as a read-only listing page. Used for the root page and
/// for any directory browsed into.
pub fn render_directory(state: &AppState, dir: &FsPath) -> String {
    let markdown = render_listing(dir, &state.base_dir);
    let file = relative_display(dir);
    let url = state.file_url(dir);
    render_page(dir, &state.base_dir, &render(&markdown, &file, &url), true)
}

fn directory_redirect(uri: &Uri) -> Option<String> {
    if uri.path().ends_with('/') {
        return None;
    }

    let mut location = format!("{}/", uri.path());
    if let Some(query) = uri.query() {
        location.push('?');
        location.push_str(query);
    }
    Some(location)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_url_gains_trailing_slash() {
        let uri = "/docs".parse().unwrap();
        assert_eq!(directory_redirect(&uri).as_deref(), Some("/docs/"));
    }

    #[test]
    fn directory_redirect_preserves_query() {
        let uri = "/docs?view=compact".parse().unwrap();
        assert_eq!(directory_redirect(&uri).as_deref(), Some("/docs/?view=compact"));
    }

    #[test]
    fn directory_url_with_trailing_slash_is_unchanged() {
        let uri = "/docs/".parse().unwrap();
        assert_eq!(directory_redirect(&uri), None);
    }
}
