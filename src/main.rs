mod args;

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;
use gh_mdp::Server;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt::layer, prelude::*, registry};

use crate::args::Args;

#[tokio::main]
async fn main() -> Result<()> {
    registry()
        .with(layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let Args { file, bind, no_open } = Args::parse();

    let file = match file {
        Some(f) if f.is_dir() => {
            let readme = f.join("README.md");
            if readme.exists() {
                info!("Directory specified, using {}", readme.display());
                readme.canonicalize().context("Failed to resolve path")?
            } else {
                bail!("README.md not found in directory: {}", f.display());
            }
        }
        Some(f) if f.exists() => f.canonicalize().context("Failed to resolve path")?,
        Some(f) => bail!("File not found: {}", f.display()),
        None => {
            let readme = PathBuf::from("README.md");
            if readme.exists() {
                info!("No file specified, using README.md");
                readme.canonicalize().context("Failed to resolve path")?
            } else {
                bail!("No file specified and README.md not found in current directory");
            }
        }
    };

    Server::try_new(file, &bind, !no_open)?.run().await
}
