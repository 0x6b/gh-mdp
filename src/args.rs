use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version)]
pub struct Args {
    /// Markdown file to preview (defaults to README.md if not specified)
    #[arg()]
    pub file: Option<PathBuf>,

    /// Port to listen on
    #[arg(short, long, default_value = "8888")]
    pub port: u16,

    /// Bind address
    #[arg(short, long, default_value = "127.0.0.1")]
    pub bind: String,

    /// Don't open browser automatically
    #[arg(long)]
    pub no_open: bool,
}
