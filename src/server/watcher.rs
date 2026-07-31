use std::{
    sync::{Arc, mpsc::channel},
    thread::{park, spawn},
    time::Duration,
};

use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use tokio::sync::mpsc::unbounded_channel;
use tracing::info;

use super::{state::AppState, util::build_gitignore};

const DEBOUNCE: Duration = Duration::from_millis(100);

pub async fn watch(state: Arc<AppState>) {
    let (tx, rx) = channel();
    let base_dir = state.base_dir.clone();

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
    let listing = state.listing;
    spawn(move || {
        while let Ok(Ok(events)) = rx.recv() {
            for e in events {
                let is_md = e.path.extension().is_some_and(|ext| ext == "md");
                let is_ignored = gitignore.matched_path_or_any_parents(&e.path, false).is_ignore();
                // Non-markdown events matter too when the root page is a directory
                // listing: adding or removing any file changes what it should show.
                if (is_md || listing) && !is_ignored {
                    let _ = notify_tx.send(e.path);
                }
            }
        }
    });

    while let Some(path) = notify_rx.recv().await {
        if path.extension().is_some_and(|ext| ext == "md") && state.refresh(&path).await {
            info!("File changed: {}", path.display());
        }
        if state.listing && state.refresh(&base_dir).await {
            info!("Directory changed: {}", base_dir.display());
        }
    }
}
