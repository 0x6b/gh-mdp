use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use gh_mdp::Server;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt::layer, prelude::*, registry};

#[derive(Parser)]
#[command(about, version)]
pub struct Args {
    /// Markdown file or directory to preview (defaults to ./index.md or ./README.md)
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
        Some(f) if f.is_dir() => resolve_markdown(&f, "Directory specified")
            .with_context(|| format!("No index.md or README.md in: {}", f.display()))?,
        Some(f) if f.exists() => f,
        Some(f) => bail!("File not found: {}", f.display()),
        None => resolve_markdown(&PathBuf::from("."), "No file specified")
            .context("No index.md or README.md found in current directory")?,
    }
    .canonicalize()
    .context("Failed to resolve path")?;

    Server::try_new(file, &bind, !no_open)?.run().await
}

fn resolve_markdown(dir: &Path, context: &str) -> Option<PathBuf> {
    ["index.md", "README.md"].into_iter().find_map(|name| {
        let path = dir.join(name);
        path.exists().then(|| {
            info!("{context}, using {name}");
            path
        })
    })
}
