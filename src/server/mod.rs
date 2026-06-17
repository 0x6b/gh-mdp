mod assets;
mod files;
mod markdown;
mod state;
mod template;
mod util;
mod watcher;
mod websocket;

use std::{
    fmt::Display,
    io,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use assets::{serve_asset, serve_favicon};
use axum::{
    Json, Router,
    extract::{Query, State},
    http::{Request, Response, StatusCode, header::CONTENT_TYPE},
    response,
    response::{Html, IntoResponse},
    routing::{get, post},
    serve,
};
use files::serve_file;
use open::that;
use serde::{Deserialize, Serialize};
use state::AppState;
use template::render_page;
use tokio::{fs::read_to_string, net::TcpListener, spawn, sync::broadcast::channel};
use tower_http::trace::TraceLayer;
use tracing::{debug, debug_span, info};
use watcher::watch;
use websocket::upgrade;

pub struct Server {
    state: Arc<AppState>,
    bind: String,
    open_browser: bool,
}

impl Server {
    pub fn try_new(file_path: PathBuf, bind: &str, open_browser: bool) -> Result<Self> {
        let (tx, _) = channel(16);
        Ok(Self {
            state: Arc::new(AppState::new(file_path, tx)),
            bind: bind.into(),
            open_browser,
        })
    }

    pub async fn run(self) -> Result<()> {
        let listener =
            TcpListener::bind(SocketAddr::from((self.bind.parse::<IpAddr>()?, 0))).await?;
        let url = format!("http://{}", listener.local_addr()?);
        self.state.set_base_url(url.clone()).await;
        info!("Listening on {url}");
        info!("Watching {}", self.state.file_path.display());

        if self.open_browser {
            let _ = that(&url);
        }

        spawn(watch(self.state.clone()));

        let app = Router::new()
            .route("/", get(serve_index))
            .route("/raw", get(serve_raw))
            .route("/save", post(save_markdown))
            .route("/toggle-task", post(toggle_task))
            .route("/favicon.ico", get(serve_favicon))
            .route("/ws", get(upgrade))
            .route("/assets/{path}", get(serve_asset))
            .route("/{*path}", get(serve_file))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|req: &Request<_>| {
                        debug_span!("request", method = %req.method(), uri = %req.uri())
                    })
                    .on_response(|res: &Response<_>, latency: Duration, _: &_| {
                        debug!(status = %res.status(), ?latency, "response");
                    }),
            )
            .with_state(self.state);

        Ok(serve(listener, app).await?)
    }
}

async fn serve_index(State(state): State<Arc<AppState>>) -> Html<String> {
    Html(render_page(&state.file_path, &state.content.read().await))
}

#[derive(Deserialize)]
struct PathQuery {
    path: Option<String>,
}

/// Resolve a target file path from an optional query/body parameter, falling back
/// to the root file. The client sends the canonical absolute path (as set by the
/// template). We verify it is a `.md` file inside the base directory.
fn resolve_target(state: &AppState, query_path: Option<&str>) -> Result<PathBuf, StatusCode> {
    match query_path {
        Some(p) => {
            let base = state
                .file_path
                .parent()
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
                .canonicalize()
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let resolved = Path::new(p).canonicalize().map_err(|_| StatusCode::NOT_FOUND)?;
            if !resolved.starts_with(&base) || resolved.extension().is_none_or(|ext| ext != "md") {
                return Err(StatusCode::FORBIDDEN);
            }
            Ok(resolved)
        }
        None => Ok(state.file_path.clone()),
    }
}

async fn serve_raw(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PathQuery>,
) -> impl IntoResponse {
    let target = match resolve_target(&state, query.path.as_deref()) {
        Ok(p) => p,
        Err(s) => return s.into_response(),
    };

    let markdown = if target == state.file_path {
        state.markdown.read().await.clone()
    } else {
        match read_to_string(&target).await {
            Ok(m) => m,
            Err(_) => return StatusCode::NOT_FOUND.into_response(),
        }
    };

    ([(CONTENT_TYPE, "text/plain; charset=utf-8")], markdown).into_response()
}

#[derive(Deserialize)]
struct SaveRequest {
    content: String,
    path: Option<String>,
}

#[derive(Serialize)]
struct SaveResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl SaveResponse {
    fn ok() -> Self {
        Self { success: true, error: None }
    }

    fn err(e: impl Display) -> Self {
        Self { success: false, error: Some(e.to_string()) }
    }
}

fn json_result(result: io::Result<()>) -> response::Response {
    match result {
        Ok(()) => Json(SaveResponse::ok()).into_response(),
        Err(e) => Json(SaveResponse::err(e)).into_response(),
    }
}

async fn save_markdown(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SaveRequest>,
) -> response::Response {
    match resolve_target(&state, payload.path.as_deref()) {
        Ok(target) => json_result(state.save(&target, &payload.content).await),
        Err(s) => s.into_response(),
    }
}

#[derive(Deserialize)]
struct ToggleTaskRequest {
    index: usize,
    path: Option<String>,
}

async fn toggle_task(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ToggleTaskRequest>,
) -> response::Response {
    match resolve_target(&state, payload.path.as_deref()) {
        Ok(target) => json_result(state.toggle_task(&target, payload.index).await),
        Err(s) => s.into_response(),
    }
}
