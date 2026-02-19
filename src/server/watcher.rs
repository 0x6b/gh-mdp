use std::{
    sync::{Arc, mpsc::channel},
    thread::{park, spawn},
    time::Duration,
};

use ignore::gitignore::Gitignore;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use tokio::sync::mpsc::unbounded_channel;
use tracing::info;

use super::state::AppState;

const DEBOUNCE: Duration = Duration::from_millis(100);

pub async fn watch(state: Arc<AppState>) {
    let (tx, rx) = channel();
    let base_dir = state.file_path.parent().unwrap_or(&state.file_path).to_path_buf();

    let gitignore = {
        let gitignore_path = base_dir.join(".gitignore");
        if gitignore_path.exists() {
            let (gi, _) = Gitignore::new(&gitignore_path);
            gi
        } else {
            Gitignore::empty()
        }
    };

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
        info!("File changed: {}", path.display());
        state.refresh(&path).await;
    }
}
