use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    fmt::Write,
    fs,
    hash::{Hash, Hasher},
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
    string,
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use serde_json::to_string;
use tokio::{
    fs::{read_to_string, write},
    sync::{RwLock, broadcast::Sender},
};

use super::{listing::render_listing, markdown::render, util::relative_display};

#[derive(Serialize)]
struct WsMessage<'a> {
    #[serde(rename = "type")]
    pub msg_type: &'a str,
    pub path: &'a str,
    pub content: &'a str,
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

fn content_hash(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// Returns the length of the bullet/marker prefix before `[ ] ` or `[x] ` if the line is a task
/// item, or 0 if it isn't. Supports `-`, `*`, `+`, and ordered list markers (e.g. `1.`).
fn task_bullet_len(trimmed: &str) -> usize {
    let checkbox =
        |s: &str| s.starts_with("[ ] ") || s.starts_with("[x] ") || s.starts_with("[X] ");

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
    /// The previewed root. Normally a markdown file; when a directory was given
    /// without an `index.md` or `README.md` it is that directory, and the root
    /// page shows a directory listing instead.
    pub file_path: PathBuf,
    /// Directory that relative links, file serving, and watching are rooted at.
    pub base_dir: PathBuf,
    /// Whether the root page is a directory listing (read-only) rather than a file.
    pub listing: bool,
    pub content: RwLock<String>,
    pub markdown: RwLock<String>,
    pub tx: Sender<String>,
    last_save_time: AtomicU64,
    /// Hash of the last content we rendered per path, used to drop redundant
    /// file-system events that report a change without the content actually
    /// changing. A single edit often produces several watcher events (e.g.
    /// editors saving via write-temp-then-rename, or flaky filesystem layers
    /// such as WSL2's `/mnt` mounts); this collapses them to one refresh.
    last_hash: RwLock<HashMap<PathBuf, u64>>,
    base_url: OnceLock<String>,
}

impl AppState {
    pub fn new(file_path: PathBuf, tx: Sender<String>) -> Self {
        let listing = file_path.is_dir();
        let base_dir = if listing {
            file_path.clone()
        } else {
            file_path.parent().unwrap_or(&file_path).to_path_buf()
        };
        let markdown = if listing {
            render_listing(&file_path, &base_dir)
        } else {
            fs::read_to_string(&file_path).unwrap_or_else(|e| format!("Error: {e}"))
        };
        Self {
            content: RwLock::new(String::new()),
            markdown: RwLock::new(markdown),
            file_path,
            base_dir,
            listing,
            tx,
            last_save_time: AtomicU64::new(0),
            last_hash: RwLock::new(HashMap::new()),
            base_url: OnceLock::new(),
        }
    }

    pub async fn set_base_url(&self, url: String) {
        let _ = self.base_url.set(url);
        let markdown = self.markdown.read().await;
        let file = relative_display(&self.file_path);
        let url = self.file_url(&self.file_path);
        *self.content.write().await = render(&markdown, &file, &url);
    }

    pub fn file_url(&self, file: &Path) -> String {
        let base = self.base_url.get().map_or("", string::String::as_str);
        let path = if file == self.file_path {
            "/".to_string()
        } else {
            file.strip_prefix(&self.base_dir)
                .map_or_else(|_| "/".to_string(), |rel| format!("/{}", rel.display()))
        };
        format!("{base}{path}")
    }

    pub async fn initial_message(&self) -> String {
        let content = self.content.read().await;
        let path = self.file_path.display().to_string();
        to_string(&WsMessage { msg_type: "update", path: &path, content: &content }).unwrap()
    }

    fn broadcast(&self, path: &Path, html: &str) {
        let path = path.display().to_string();
        let _ = self.tx.send(
            to_string(&WsMessage { msg_type: "update", path: &path, content: html }).unwrap(),
        );
    }

    /// Re-render and broadcast the given path. Returns `true` when an update was
    /// actually sent, and `false` when the event was dropped as redundant (our
    /// own save, or unchanged content) so callers can avoid logging no-op events.
    pub async fn refresh(&self, changed_path: &Path) -> bool {
        // Skip if this change was caused by our own save (infinite loop prevention)
        let last_save = self.last_save_time.load(Ordering::SeqCst);
        if now_millis().saturating_sub(last_save) < 500 {
            return false;
        }

        let markdown = if changed_path.is_dir() {
            render_listing(changed_path, &self.base_dir)
        } else {
            read_to_string(changed_path)
                .await
                .unwrap_or_else(|e| format!("Error: {e}"))
        };

        // Drop spurious events: if the content is identical to what we last
        // rendered for this path, there is nothing to refresh.
        let hash = content_hash(&markdown);
        {
            let mut last_hash = self.last_hash.write().await;
            if last_hash.get(changed_path) == Some(&hash) {
                return false;
            }
            last_hash.insert(changed_path.to_path_buf(), hash);
        }

        let file = relative_display(changed_path);
        let url = self.file_url(changed_path);
        let html = render(&markdown, &file, &url);

        if changed_path == self.file_path {
            *self.content.write().await = html.clone();
            *self.markdown.write().await = markdown;
        }
        self.broadcast(changed_path, &html);
        true
    }

    pub async fn save(&self, target: &Path, content: &str) -> Result<()> {
        // Record save time before writing (infinite loop prevention)
        self.last_save_time.store(now_millis(), Ordering::SeqCst);

        write(target, content).await?;

        // Record the saved content's hash so the resulting watcher event is
        // recognised as a no-op change and does not trigger a redundant refresh.
        self.last_hash
            .write()
            .await
            .insert(target.to_path_buf(), content_hash(content));

        let file = relative_display(target);
        let url = self.file_url(target);
        let html = render(content, &file, &url);

        // Update cached state only for the root file
        if target == self.file_path {
            *self.markdown.write().await = content.to_string();
            *self.content.write().await = html.clone();
        }

        // Broadcast update so preview stays in sync
        self.broadcast(target, &html);

        Ok(())
    }

    pub async fn toggle_task(&self, target: &Path, index: usize) -> Result<()> {
        let markdown = if target == self.file_path {
            self.markdown.read().await.clone()
        } else {
            read_to_string(target)
                .await
                .map_err(|e| Error::new(ErrorKind::NotFound, e))?
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
                    let marker = if trimmed[bullet_len..bullet_len + 4].eq_ignore_ascii_case("[x] ")
                    {
                        "[ ] "
                    } else {
                        "[x] "
                    };
                    write!(result, "{indent}{bullet}{marker}{rest}").unwrap();
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
