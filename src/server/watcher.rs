use std::{
    sync::{Arc, mpsc::channel},
    thread::{park, spawn},
    time::Duration,
};

use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use tokio::sync::mpsc::unbounded_channel;
use tracing::info;

use super::state::AppState;

/// Debounce duration for file system events to avoid excessive refreshes.
const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

pub async fn watch(state: Arc<AppState>) {
    let (tx, rx) = channel();
    let file_path = state.file_path.clone();
    let parent_dir = file_path.parent().unwrap_or(&file_path).to_path_buf();

    spawn(move || {
        let mut debouncer =
            new_debouncer(DEBOUNCE_DURATION, tx).expect("Failed to create debouncer");

        debouncer
            .watcher()
            .watch(&parent_dir, RecursiveMode::NonRecursive)
            .expect("Failed to watch directory");

        park();
    });

    let (notify_tx, mut notify_rx) = unbounded_channel();
    let target_file = state.file_path.clone();

    spawn(move || {
        while let Ok(Ok(events)) = rx.recv() {
            if events.iter().any(|e| e.path == target_file) {
                let _ = notify_tx.send(());
            }
        }
    });

    while notify_rx.recv().await.is_some() {
        info!("File changed: {}", state.file_path.display());
        state.refresh().await;
    }
}
