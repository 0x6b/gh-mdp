use std::{
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use serde_json::to_string;
use tokio::{
    fs::{read_to_string, write},
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

/// Returns the length of the bullet/marker prefix before `[ ] ` or `[x] ` if the line is a task
/// item, or 0 if it isn't. Supports `-`, `*`, `+`, and ordered list markers (e.g. `1.`).
fn task_bullet_len(trimmed: &str) -> usize {
    let checkbox = |s: &str| s.starts_with("[ ] ") || s.starts_with("[x] ") || s.starts_with("[X] ");

    // Unordered: "- ", "* ", "+ " followed by checkbox
    if let Some(rest) = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
        .or_else(|| trimmed.strip_prefix("+ "))
        && checkbox(rest)
    {
        return 2; // "- " / "* " / "+ "
    }

    // Ordered: digits followed by ". " or ") " then checkbox
    let digit_end = trimmed.find(|c: char| !c.is_ascii_digit()).unwrap_or(0);
    if digit_end > 0 {
        let after_digits = &trimmed[digit_end..];
        if let Some(rest) = after_digits
            .strip_prefix(". ")
            .or_else(|| after_digits.strip_prefix(") "))
            && checkbox(rest)
        {
            return digit_end + 2; // "1. " / "1) "
        }
    }

    0
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
        let markdown =
            std::fs::read_to_string(&file_path).unwrap_or_else(|e| format!("Error: {e}"));
        let content = render(&markdown);
        Self {
            content: RwLock::new(content),
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

        let markdown = read_to_string(changed_path)
            .await
            .unwrap_or_else(|e| format!("Error: {e}"));
        let html = render(&markdown);

        if changed_path == self.file_path {
            *self.content.write().await = html.clone();
            *self.markdown.write().await = markdown;
        }
        let path = changed_path.display().to_string();
        let _ = self.tx.send(
            to_string(&WsMessage { msg_type: "update", path: &path, content: &html }).unwrap(),
        );
    }

    pub async fn save(&self, target: &Path, content: &str) -> Result<()> {
        // Record save time before writing (infinite loop prevention)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.last_save_time.store(now, Ordering::SeqCst);

        write(target, content).await?;

        let html = render(content);

        // Update cached state only for the root file
        if target == self.file_path {
            *self.markdown.write().await = content.to_string();
            *self.content.write().await = html.clone();
        }

        // Broadcast update so preview stays in sync
        let path = target.display().to_string();
        let _ = self.tx.send(
            to_string(&WsMessage { msg_type: "update", path: &path, content: &html }).unwrap(),
        );

        Ok(())
    }

    pub async fn toggle_task(&self, target: &Path, index: usize) -> Result<()> {
        let markdown = if target == self.file_path {
            self.markdown.read().await.clone()
        } else {
            read_to_string(target).await.map_err(|e| Error::new(ErrorKind::NotFound, e))?
        };
        let mut task_count = 0;
        let mut result = String::with_capacity(markdown.len());
        let mut toggled = false;

        for line in markdown.lines() {
            let trimmed = line.trim_start();
            let bullet_len = task_bullet_len(trimmed);

            if bullet_len > 0 {
                if task_count == index {
                    let indent = &line[..line.len() - trimmed.len()];
                    let bullet = &trimmed[..bullet_len];
                    let rest = &trimmed[bullet_len + 4..];
                    let marker = if trimmed[bullet_len..bullet_len + 4]
                        .eq_ignore_ascii_case("[x] ")
                    {
                        "[ ] "
                    } else {
                        "[x] "
                    };
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

        self.save(target, &result).await
    }
}
