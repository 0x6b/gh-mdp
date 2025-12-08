use std::{error::Error, fs::File, io::copy, path::Path};

use reqwest::blocking::Client;

const ASSETS: &[(&str, &str)] = &[
    // highlight.js (BSD-3-Clause)
    (
        "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/highlight.min.js",
        "highlight.min.js",
    ),
    (
        "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/styles/github.min.css",
        "highlight-github.min.css",
    ),
    (
        "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/styles/github-dark.min.css",
        "highlight-github-dark.min.css",
    ),
    (
        "https://raw.githubusercontent.com/highlightjs/highlight.js/main/LICENSE",
        "LICENSE-highlight.js",
    ),
    // morphdom (MIT)
    ("https://unpkg.com/morphdom@2.7.7/dist/morphdom-umd.min.js", "morphdom.min.js"),
    (
        "https://raw.githubusercontent.com/patrick-steele-idem/morphdom/master/LICENSE",
        "LICENSE-morphdom",
    ),
    // github-markdown-css (MIT)
    (
        "https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.8.1/github-markdown.min.css",
        "github-markdown.min.css",
    ),
    (
        "https://raw.githubusercontent.com/sindresorhus/github-markdown-css/main/license",
        "LICENSE-github-markdown-css",
    ),
    // mermaid (MIT)
    ("https://cdn.jsdelivr.net/npm/mermaid@11.12.2/dist/mermaid.min.js", "mermaid.min.js"),
    ("https://raw.githubusercontent.com/mermaid-js/mermaid/develop/LICENSE", "LICENSE-mermaid"),
];

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let assets_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");

    // Tell cargo to rerun only if asset files are missing or changed
    for (_, filename) in ASSETS {
        println!("cargo:rerun-if-changed=assets/{filename}");
    }

    let missing: Vec<_> = ASSETS
        .iter()
        .filter(|(_, filename)| !assets_root.join(filename).exists())
        .collect();

    if missing.is_empty() {
        return Ok(());
    }

    let client = Client::builder()
        .user_agent("markdown-preview/0.1.0 (https://github.com/user/markdown-preview)")
        .build()?;

    for (url, filename) in missing {
        let path = assets_root.join(filename);
        println!("cargo:warning=Downloading {filename}");
        let blob = client.get(*url).send()?.bytes()?;
        let mut out = File::create(&path)?;
        copy(&mut blob.as_ref(), &mut out)?;
    }

    Ok(())
}
