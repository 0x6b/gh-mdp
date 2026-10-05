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
    template::{render_html_page, render_page},
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

    if is_html_file(&resolved) && !is_raw_html_request(&uri) {
        let raw_url = raw_html_url(&state.url_path(&resolved), &uri);
        return Html(render_html_page(&resolved, &state.base_dir, &raw_url)).into_response();
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

pub(super) fn is_html_file(path: &FsPath) -> bool {
    path.extension().is_some_and(|extension| {
        extension.eq_ignore_ascii_case("html")
            || extension.eq_ignore_ascii_case("htm")
            || extension.eq_ignore_ascii_case("xhtml")
            || extension.eq_ignore_ascii_case("xht")
    })
}

fn is_raw_html_request(uri: &Uri) -> bool {
    uri.query()
        .is_some_and(|query| query.split('&').any(|param| param == "__gh_mdp_raw=1"))
}

fn raw_html_url(path: &str, uri: &Uri) -> String {
    match uri.query() {
        Some(query) => format!("{path}?{query}&__gh_mdp_raw=1"),
        None => format!("{path}?__gh_mdp_raw=1"),
    }
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

    #[test]
    fn html_file_extensions_include_xhtml_but_not_similar_names() {
        assert!(is_html_file(FsPath::new("preview.HTML")));
        assert!(is_html_file(FsPath::new("preview.htm")));
        assert!(is_html_file(FsPath::new("preview.xhtml")));
        assert!(is_html_file(FsPath::new("preview.XHT")));
        assert!(!is_html_file(FsPath::new("preview.xhtm")));
    }

    #[test]
    fn raw_html_query_is_an_exact_parameter() {
        assert!(is_raw_html_request(&"/preview.html?__gh_mdp_raw=1".parse().unwrap()));
        assert!(is_raw_html_request(&"/preview.html?theme=dark&__gh_mdp_raw=1".parse().unwrap()));
        assert!(!is_raw_html_request(&"/preview.html?__gh_mdp_raw=10".parse().unwrap()));
        assert!(!is_raw_html_request(&"/preview.html?raw=1".parse().unwrap()));
    }

    #[test]
    fn raw_html_url_preserves_the_page_query() {
        assert_eq!(
            raw_html_url("/preview.html", &"/preview.html?theme=dark".parse().unwrap()),
            "/preview.html?theme=dark&__gh_mdp_raw=1"
        );
        assert_eq!(
            raw_html_url("/preview.html", &"/preview.html".parse().unwrap()),
            "/preview.html?__gh_mdp_raw=1"
        );
    }
}
