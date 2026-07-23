use std::{
    path::Path,
    sync::{Arc, mpsc::channel},
    thread::{park, spawn},
    time::Duration,
};

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use tokio::sync::mpsc::unbounded_channel;
use tracing::info;

use super::state::AppState;

const DEBOUNCE: Duration = Duration::from_millis(100);

/// Build a composite Gitignore by collecting `.gitignore` files from `base_dir` up to the
/// filesystem root. Files are added from the outermost ancestor first so that closer rules
/// take precedence, matching standard git behavior.
fn build_gitignore(base_dir: &Path) -> Gitignore {
    let mut paths = Vec::new();
    let mut dir = Some(base_dir);
    while let Some(d) = dir {
        let gi = d.join(".gitignore");
        if gi.exists() {
            paths.push(gi);
        }
        dir = d.parent();
    }

    if paths.is_empty() {
        return Gitignore::empty();
    }

    // Add outermost first so inner rules override
    let mut builder = GitignoreBuilder::new(base_dir);
    for path in paths.into_iter().rev() {
        let _ = builder.add(path);
    }
    builder.build().unwrap_or_else(|_| Gitignore::empty())
}

pub async fn watch(state: Arc<AppState>) {
    let (tx, rx) = channel();
    let base_dir = state.file_path.parent().unwrap_or(&state.file_path).to_path_buf();

    let gitignore = build_gitignore(&base_dir);

    info!("Watching for markdown changes in {}", base_dir.display());

    spawn({
        let base_dir = base_dir.clone();
        move || {
            let mut debouncer = new_debouncer(DEBOUNCE, tx).expect("Failed to create debouncer");
            debouncer
                .watcher()
                .watch(&base_dir, RecursiveMode::Recursive)
                .expect("Failed to watch");
            park();
        }
    });

    let (notify_tx, mut notify_rx) = unbounded_channel();
    spawn(move || {
        while let Ok(Ok(events)) = rx.recv() {
            for e in events {
                let is_md = e.path.extension().is_some_and(|ext| ext == "md");
                let is_ignored = gitignore.matched_path_or_any_parents(&e.path, false).is_ignore();
                if is_md && !is_ignored {
                    let _ = notify_tx.send(e.path);
                }
            }
        }
    });

    while let Some(path) = notify_rx.recv().await {
        if state.refresh(&path).await {
            info!("File changed: {}", path.display());
        }
    }
}
