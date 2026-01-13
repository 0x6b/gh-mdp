mod assets;
mod files;
mod markdown;
mod state;
mod template;
mod util;
mod watcher;
mod websocket;

use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::{Request, Response, header::CONTENT_TYPE},
    response::{Html, IntoResponse},
    routing::{get, post},
    serve,
};
use open::that;
use serde::{Deserialize, Serialize};
use state::AppState;
use template::render_page;
use tokio::{net::TcpListener, spawn, sync::broadcast::channel};
use tower_http::trace::TraceLayer;
use tracing::{info, info_span};
use watcher::watch;

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
            .route("/favicon.ico", get(assets::serve_favicon))
            .route("/ws", get(websocket::upgrade))
            .route("/assets/{path}", get(assets::serve_asset))
            .route("/{*path}", get(files::serve_file))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|req: &Request<_>| {
                        info_span!("request", method = %req.method(), uri = %req.uri())
                    })
                    .on_response(|res: &Response<_>, latency: Duration, _: &_| {
                        info!(status = %res.status(), ?latency, "response");
                    }),
            )
            .with_state(self.state);

        Ok(serve(listener, app).await?)
    }
}

async fn serve_index(State(state): State<Arc<AppState>>) -> Html<String> {
    Html(render_page(&state.file_path, &state.content.read().await))
}

async fn serve_raw(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let markdown = state.markdown.read().await;
    ([(CONTENT_TYPE, "text/plain; charset=utf-8")], markdown.clone())
}

#[derive(Deserialize)]
struct SaveRequest {
    content: String,
}

#[derive(Serialize)]
struct SaveResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn save_markdown(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SaveRequest>,
) -> Json<SaveResponse> {
    match state.save(&payload.content).await {
        Ok(()) => Json(SaveResponse { success: true, error: None }),
        Err(e) => Json(SaveResponse { success: false, error: Some(e.to_string()) }),
    }
}
