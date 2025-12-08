mod args;

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;
use gh_mdp::Server;
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, fmt::layer, prelude::*, registry};

use crate::args::Args;

#[tokio::main]
async fn main() -> Result<()> {
    registry()
        .with(layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let Args { file, port, bind, no_open } = Args::parse();

    let file = match file {
        Some(f) => f,
        None => {
            let readme = PathBuf::from("README.md");
            if readme.exists() {
                info!("No file specified, using README.md");
                readme
            } else {
                warn!("No file specified and README.md not found in current directory");
                bail!("No file specified and README.md not found in current directory");
            }
        }
    };

    if !file.exists() {
        bail!("File not found: {}", file.display());
    }

    let file_path = file.canonicalize().context("Failed to resolve path")?;
    let server = Server::try_new(file_path, &bind, port, !no_open)?;

    server.run().await
}
