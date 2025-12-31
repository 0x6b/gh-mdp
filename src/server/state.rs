use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::to_string;
use tokio::sync::{RwLock, broadcast::Sender};

use super::markdown::render;

#[derive(Serialize)]
pub struct WsMessage<'a> {
    #[serde(rename = "type")]
    pub msg_type: &'a str,
    pub path: &'a str,
    pub content: &'a str,
}

pub struct AppState {
    pub file_path: PathBuf,
    pub content: RwLock<String>,
    pub tx: Sender<String>,
}

impl AppState {
    pub fn new(file_path: PathBuf, tx: Sender<String>) -> Self {
        Self {
            content: RwLock::new(render(&file_path)),
            file_path,
            tx,
        }
    }

    pub async fn refresh(&self, changed_path: &Path) {
        let html = render(changed_path);
        if changed_path == self.file_path {
            *self.content.write().await = html.clone();
        }
        let path = changed_path.display().to_string();
        let _ = self.tx.send(
            to_string(&WsMessage { msg_type: "update", path: &path, content: &html }).unwrap(),
        );
    }
}
