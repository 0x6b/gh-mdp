use std::{
    fs::read_to_string,
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use serde_json::to_string;
use tokio::{
    fs::write,
    sync::{RwLock, broadcast::Sender},
};

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
    pub markdown: RwLock<String>,
    pub tx: Sender<String>,
    last_save_time: AtomicU64,
}

impl AppState {
    pub fn new(file_path: PathBuf, tx: Sender<String>) -> Self {
        let markdown = read_to_string(&file_path).unwrap_or_else(|e| format!("Error: {e}"));
        Self {
            content: RwLock::new(render(&file_path)),
            markdown: RwLock::new(markdown),
            file_path,
            tx,
            last_save_time: AtomicU64::new(0),
        }
    }

    pub async fn refresh(&self, changed_path: &Path) {
        // Skip if this change was caused by our own save (infinite loop prevention)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let last_save = self.last_save_time.load(Ordering::SeqCst);
        if now.saturating_sub(last_save) < 500 {
            return;
        }

        let html = render(changed_path);
        let markdown = read_to_string(changed_path).unwrap_or_else(|e| format!("Error: {e}"));

        if changed_path == self.file_path {
            *self.content.write().await = html.clone();
            *self.markdown.write().await = markdown;
        }
        let path = changed_path.display().to_string();
        let _ = self.tx.send(
            to_string(&WsMessage { msg_type: "update", path: &path, content: &html }).unwrap(),
        );
    }

    pub async fn save(&self, content: &str) -> Result<()> {
        // Record save time before writing (infinite loop prevention)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.last_save_time.store(now, Ordering::SeqCst);

        write(&self.file_path, content).await?;

        // Update internal state
        *self.markdown.write().await = content.to_string();
        let html = render(&self.file_path);
        *self.content.write().await = html.clone();

        // Broadcast update so preview stays in sync
        let path = self.file_path.display().to_string();
        let _ = self.tx.send(
            to_string(&WsMessage { msg_type: "update", path: &path, content: &html }).unwrap(),
        );

        Ok(())
    }

    pub async fn toggle_task(&self, index: usize) -> Result<()> {
        let markdown = self.markdown.read().await.clone();
        let mut task_count = 0;
        let mut result = String::with_capacity(markdown.len());
        let mut toggled = false;

        for line in markdown.lines() {
            let trimmed = line.trim_start();
            let is_task = (trimmed.starts_with("- [ ] ") || trimmed.starts_with("- [x] "))
                || (trimmed.starts_with("* [ ] ") || trimmed.starts_with("* [x] "));

            if is_task {
                if task_count == index {
                    let indent = &line[..line.len() - trimmed.len()];
                    let bullet = &trimmed[..2];
                    let rest = &trimmed[6..];
                    let marker =
                        if trimmed[3..4].eq_ignore_ascii_case("x") { "[ ] " } else { "[x] " };
                    result.push_str(indent);
                    result.push_str(bullet);
                    result.push_str(marker);
                    result.push_str(rest);
                    toggled = true;
                } else {
                    result.push_str(line);
                }
                task_count += 1;
            } else {
                result.push_str(line);
            }
            result.push('\n');
        }

        if !toggled {
            return Err(Error::new(ErrorKind::NotFound, "task item not found"));
        }

        // Trim trailing newline if original didn't end with one
        if !markdown.ends_with('\n') {
            result.pop();
        }

        self.save(&result).await
    }
}
