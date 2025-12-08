use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, version)]
pub struct Args {
    /// Markdown file or directory to preview (defaults to index.md or README.md)
    #[arg()]
    pub file: Option<PathBuf>,

    /// Bind address
    #[arg(short, long, default_value = "127.0.0.1")]
    pub bind: String,

    /// Don't open browser automatically
    #[arg(long)]
    pub no_open: bool,
}
