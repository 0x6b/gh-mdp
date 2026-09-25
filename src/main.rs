use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use gh_mdp::{Server, default_markdown};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt::layer, prelude::*, registry};

const LICENSES: &str = concat!(
    "gh-mdp\n======\n",
    include_str!("../LICENSE"),
    "\n\ngithub-markdown-css\n===================\n",
    include_str!("../assets/LICENSE-github-markdown-css"),
    "\n\nhighlight.js\n============\n",
    include_str!("../assets/LICENSE-highlight.js"),
    "\n\nMermaid\n=======\n",
    include_str!("../assets/LICENSE-mermaid"),
    "\n\nmorphdom\n========\n",
    include_str!("../assets/LICENSE-morphdom"),
    "\n\nOverType\n========\n",
    include_str!("../assets/LICENSE-overtype"),
    "\n\nOcticons\n========\n",
    include_str!("../assets/LICENSE-octicons"),
    "\n",
);

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
    /// Print licenses and exit
    #[arg(long)]
    pub licenses: bool,
    /// Open the preview in the gh-mdp app
    #[arg(long, hide = true)]
    pub app: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    registry()
        .with(layer())
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let Args { file, bind, no_open, licenses, app } = Args::parse();

    if licenses {
        print!("{LICENSES}");
        return Ok(());
    }

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

    if app {
        #[cfg(windows)]
        return run_app(file, &bind).await;

        #[cfg(not(windows))]
        bail!("The gh-mdp app is currently available on Windows only");
    }

    Server::try_new(file, &bind, !no_open)?.run().await
}

#[cfg(windows)]
async fn run_app(file: PathBuf, bind: &str) -> Result<()> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};
    use windows_sys::Win32::System::Console::FreeConsole;

    // This binary keeps the console subsystem for `gh mdp`. Explorer creates a
    // console when a file association starts it, so detach only in app mode.
    unsafe {
        FreeConsole();
    }

    let title = format!("{} - gh-mdp", file.display());
    let base_dir =
        if file.is_dir() { file.clone() } else { file.parent().unwrap_or(&file).to_path_buf() };
    let server = Server::try_new(file, bind, false)?.bind().await?;
    let page_url: tauri::Url = server.url().parse()?;
    let server_origin = page_url.origin().ascii_serialization();
    let server_task = tokio::spawn(server.run());

    tauri::Builder::default()
        .setup(move |app| {
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(page_url))
                .title(title)
                .inner_size(1200.0, 800.0)
                .on_navigation(move |url| {
                    if url.origin().ascii_serialization() != server_origin {
                        let _ = open::that(url.as_str());
                        return false;
                    }

                    let Some(path) = resolve_app_path(&base_dir, url.path()) else {
                        return true;
                    };
                    if path.is_file() && path.extension().is_none_or(|extension| extension != "md")
                    {
                        let _ = open::that(path);
                        return false;
                    }

                    true
                })
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())?;

    server_task.abort();
    Ok(())
}

#[cfg(any(windows, test))]
fn resolve_app_path(base_dir: &Path, url_path: &str) -> Option<PathBuf> {
    let decoded = percent_encoding::percent_decode_str(url_path).decode_utf8().ok()?;
    let resolved = base_dir
        .join(decoded.strip_prefix('/').unwrap_or(&decoded))
        .canonicalize()
        .ok()?;
    resolved.starts_with(base_dir).then_some(resolved)
}

/// Find the markdown file to preview inside `dir`. Returns `None` when the directory
/// has neither, in which case the directory itself is previewed as a file listing.
fn resolve_markdown(dir: &Path, context: &str) -> Option<PathBuf> {
    if let Some(path) = default_markdown(dir) {
        info!("{context}, using {}", path.display());
        Some(path)
    } else {
        info!("{context}, no index.md or README.md; showing directory listing");
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_paths_are_decoded_and_confined_to_the_served_directory() {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize().unwrap();
        let readme = base.join("README.md").canonicalize().unwrap();

        assert_eq!(resolve_app_path(&base, "/README%2Emd"), Some(readme));
        assert_eq!(resolve_app_path(&base, "/%2E%2E"), None);
    }
}
