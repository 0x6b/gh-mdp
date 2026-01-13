use std::{error::Error, fs::File, io::copy, path::Path};

use reqwest::blocking::Client;

const ASSETS: &[(&str, &str)] = &[
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
    ("https://unpkg.com/morphdom@2.7.7/dist/morphdom-umd.min.js", "morphdom.min.js"),
    (
        "https://raw.githubusercontent.com/patrick-steele-idem/morphdom/master/LICENSE",
        "LICENSE-morphdom",
    ),
    (
        "https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.8.1/github-markdown.min.css",
        "github-markdown.min.css",
    ),
    (
        "https://raw.githubusercontent.com/sindresorhus/github-markdown-css/main/license",
        "LICENSE-github-markdown-css",
    ),
    ("https://cdn.jsdelivr.net/npm/mermaid@11.12.2/dist/mermaid.min.js", "mermaid.min.js"),
    ("https://raw.githubusercontent.com/mermaid-js/mermaid/develop/LICENSE", "LICENSE-mermaid"),
    ("https://unpkg.com/overtype@2.1.1/dist/overtype.min.js", "overtype.min.js"),
    ("https://raw.githubusercontent.com/panphora/overtype/main/LICENSE", "LICENSE-overtype"),
];

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    let assets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");

    for (_, name) in ASSETS {
        println!("cargo:rerun-if-changed=assets/{name}");
    }

    let missing: Vec<_> = ASSETS
        .iter()
        .filter(|(_, name)| !assets_dir.join(name).exists())
        .collect();
    if missing.is_empty() {
        return Ok(());
    }

    let client = Client::builder().user_agent("gh-mdp").build()?;
    for (url, name) in missing {
        println!("cargo:warning=Downloading {name}");
        let blob = client.get(*url).send()?.bytes()?;
        copy(&mut blob.as_ref(), &mut File::create(assets_dir.join(name))?)?;
    }
    Ok(())
}
