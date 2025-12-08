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
    Router,
    extract::State,
    http::{Request, Response},
    response::Html,
    routing::get,
    serve,
};
use open::that;
use state::AppState;
use tokio::{net::TcpListener, spawn, sync::broadcast::channel};
use tower_http::trace::TraceLayer;
use tracing::{Span, info, info_span};
use watcher::watch;

pub struct Server {
    state: Arc<AppState>,
    bind: String,
    open_browser: bool,
}

impl Server {
    pub fn try_new(file_path: PathBuf, bind: &str, open_browser: bool) -> Result<Self> {
        let (tx, _rx) = channel(16);
        let state = Arc::new(AppState::new(file_path, tx));
        Ok(Self { state, bind: bind.to_string(), open_browser })
    }

    pub async fn run(self) -> Result<()> {
        let listener =
            TcpListener::bind(SocketAddr::from((self.bind.parse::<IpAddr>()?, 0))).await?;
        let addr = listener.local_addr()?;
        let url = format!("http://{addr}");
        info!("Listening on {url}");
        info!("Watching {}", self.state.file_path.display());

        if self.open_browser {
            let _ = that(&url);
        }

        let watcher_state = self.state.clone();
        spawn(async move {
            watch(watcher_state).await;
        });

        let app = Router::new()
            .route("/", get(serve_index))
            .route("/favicon.ico", get(assets::serve_favicon))
            .route("/ws", get(websocket::upgrade))
            .route("/assets/{path}", get(assets::serve_asset))
            .route("/{*path}", get(files::serve_file))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request<_>| {
                        info_span!(
                            "request",
                            method = %request.method(),
                            uri = %request.uri(),
                        )
                    })
                    .on_response(|response: &Response<_>, latency: Duration, _span: &Span| {
                        info!(status = %response.status(), latency = ?latency, "response");
                    }),
            )
            .with_state(self.state);

        serve(listener, app).await?;

        Ok(())
    }
}

async fn serve_index(State(state): State<Arc<AppState>>) -> Html<String> {
    let content = state.content.read().await.clone();
    Html(template::render_page(&state.file_path, &content))
}
