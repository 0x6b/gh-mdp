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
        let content = render(&file_path);

        Self { file_path, content: RwLock::new(content), tx }
    }

    pub async fn refresh(&self, changed_path: &Path) {
        let html = render(changed_path);
        let path = changed_path.display().to_string();

        // Update stored content only for the main file
        if changed_path == self.file_path {
            let mut content = self.content.write().await;
            *content = html.clone();
        }

        let msg = to_string(&WsMessage {
            msg_type: "update",
            path: &path,
            content: &html,
        })
        .unwrap();
        let _ = self.tx.send(msg);
    }
}
