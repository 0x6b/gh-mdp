use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use gh_mdp::Server;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt::layer, prelude::*, registry};

#[derive(Parser)]
#[command(about, version)]
pub struct Args {
    /// Markdown file or directory to preview (defaults to ./index.md, ./README.md, or a
    /// listing of the current directory)
    pub file: Option<PathBuf>,
    /// Bind address
    #[arg(short, long, default_value = "127.0.0.1")]
    pub bind: String,
    /// Don't open browser automatically
    #[arg(long)]
    pub no_open: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    registry()
        .with(layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let Args { file, bind, no_open } = Args::parse();

    let file = match file {
        Some(f) if f.is_dir() => resolve_markdown(&f, "Directory specified").unwrap_or(f),
        Some(f) if f.exists() => f,
        Some(f) => bail!("File not found: {}", f.display()),
        None => {
            let cwd = PathBuf::from(".");
            resolve_markdown(&cwd, "No file specified").unwrap_or(cwd)
        }
    }
    .canonicalize()
    .context("Failed to resolve path")?;

    Server::try_new(file, &bind, !no_open)?.run().await
}

/// Find the markdown file to preview inside `dir`. Returns `None` when the directory
/// has neither, in which case the directory itself is previewed as a file listing.
fn resolve_markdown(dir: &Path, context: &str) -> Option<PathBuf> {
    let found = ["index.md", "README.md"].into_iter().find_map(|name| {
        let path = dir.join(name);
        path.exists().then(|| {
            info!("{context}, using {name}");
            path
        })
    });
    if found.is_none() {
        info!("{context}, no index.md or README.md; showing directory listing");
    }
    found
}
