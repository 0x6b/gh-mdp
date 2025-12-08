mod assets;
mod markdown;
mod state;
mod watcher;
mod websocket;

use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Result;
use assets::render_page;
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
    addr: SocketAddr,
    open_browser: bool,
}

impl Server {
    pub fn try_new(file_path: PathBuf, bind: &str, port: u16, open_browser: bool) -> Result<Self> {
        let (tx, _rx) = channel(16);
        let state = Arc::new(AppState::new(file_path, tx));
        let addr = format!("{bind}:{port}").parse()?;
        Ok(Self { state, addr, open_browser })
    }

    pub async fn run(self) -> Result<()> {
        let url = format!("http://{}", self.addr);
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
            .route("/", get(index_handler))
            .route("/ws", get(websocket::handler))
            .route("/assets/{path}", get(assets::handler))
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

        serve(TcpListener::bind(self.addr).await?, app).await?;

        Ok(())
    }
}

async fn index_handler(State(state): State<Arc<AppState>>) -> Html<String> {
    let content = state.content.read().await.clone();
    Html(render_page(&state.file_path, &content))
}
