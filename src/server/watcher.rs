use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, mpsc::channel},
    thread::{park, spawn},
    time::Duration,
};

use ignore::WalkBuilder;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use tokio::sync::mpsc::unbounded_channel;
use tracing::info;

use super::state::AppState;

const DEBOUNCE: Duration = Duration::from_millis(100);

pub async fn watch(state: Arc<AppState>) {
    let (tx, rx) = channel();
    let base_dir = state.file_path.parent().unwrap_or(&state.file_path).to_path_buf();

    let md_files: HashSet<PathBuf> = WalkBuilder::new(&base_dir)
        .build()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.into_path())
        .collect();

    let dirs: HashSet<_> = md_files
        .iter()
        .filter_map(|p| p.parent().map(PathBuf::from))
        .collect();

    info!("Watching {} markdown files in {} directories", md_files.len(), dirs.len());

    spawn(move || {
        let mut debouncer = new_debouncer(DEBOUNCE, tx).expect("Failed to create debouncer");
        for dir in &dirs {
            debouncer
                .watcher()
                .watch(dir, RecursiveMode::NonRecursive)
                .expect("Failed to watch");
        }
        park();
    });

    let (notify_tx, mut notify_rx) = unbounded_channel();
    spawn(move || {
        while let Ok(Ok(events)) = rx.recv() {
            for e in events.into_iter().filter(|e| md_files.contains(&e.path)) {
                let _ = notify_tx.send(e.path);
            }
        }
    });

    while let Some(path) = notify_rx.recv().await {
        info!("File changed: {}", path.display());
        state.refresh(&path).await;
    }
}
