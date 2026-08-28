use std::{error::Error, fs, io, path::Path};

use reqwest::blocking::Client;
use sha2::{Digest, Sha256};

struct Asset {
    url: &'static str,
    name: &'static str,
    sha256: &'static str,
}

const ASSETS: &[Asset] = &[
    Asset {
        url: "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/highlight.min.js",
        name: "highlight.min.js",
        sha256: "c4a399dd6f488bc97a3546e3476747b3e714c99c57b9473154c6fb8d259b9381",
    },
    Asset {
        url: "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/styles/github.min.css",
        name: "highlight-github.min.css",
        sha256: "3a9a5def8b9c311e5ae43abde85c63133185eed4f0d9f67fea4b00a8308cf066",
    },
    Asset {
        url: "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/styles/github-dark.min.css",
        name: "highlight-github-dark.min.css",
        sha256: "9f208d022102b1d0c7aebfecd8e42ca7997d5de636649d2b31ea63093d809019",
    },
    Asset {
        url: "https://raw.githubusercontent.com/highlightjs/highlight.js/08cb242e7d4aee787114eb04cc7ab18314d82f92/LICENSE",
        name: "LICENSE-highlight.js",
        sha256: "6c081431591d9df696c82dc598fe1423765b8a299b200ed00b281afd0f64c490",
    },
    Asset {
        url: "https://cdn.jsdelivr.net/npm/morphdom@2.7.7/dist/morphdom-umd.min.js",
        name: "morphdom.min.js",
        sha256: "ad1aaf5441eb2798b99dd03a41bea26562cd634dfc9845d3b8c5fbce560a6bac",
    },
    Asset {
        url: "https://raw.githubusercontent.com/patrick-steele-idem/morphdom/a87fc2beea71308d96d228c51ed7c0949a91a492/LICENSE",
        name: "LICENSE-morphdom",
        sha256: "dc18a39627c8f2e5391255635bd1d6bbb02e91ba4a9bd3e13e53347b8c25e61a",
    },
    Asset {
        url: "https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.8.1/github-markdown.min.css",
        name: "github-markdown.min.css",
        sha256: "c47f5a601c095973e19c0a7d0418d35b2b209098955d2cc4136eb274f9083cc4",
    },
    Asset {
        url: "https://raw.githubusercontent.com/sindresorhus/github-markdown-css/e771b613e93f868afd7ce2cdba2a2c7b6c649416/license",
        name: "LICENSE-github-markdown-css",
        sha256: "5c932d88256b4ab958f64a856fa48e8bd1f55bc1d96b8149c65689e0c61789d3",
    },
    Asset {
        url: "https://cdn.jsdelivr.net/npm/mermaid@11.17.2/dist/mermaid.min.js",
        name: "mermaid.min.js",
        sha256: "581ed7d74bd9048d0e3a91363927d72ef22942d7722546b27f7cc29e35390eb8",
    },
    Asset {
        url: "https://raw.githubusercontent.com/mermaid-js/mermaid/dcb694ddb58dc5ad3502e7e903cac05fd812eac3/LICENSE",
        name: "LICENSE-mermaid",
        sha256: "ec9fb67dcb25eccc416ed56e1aab819222c805a2a4bfe4cb19e7556bf2ffde80",
    },
    Asset {
        url: "https://cdn.jsdelivr.net/npm/overtype@2.4.0/dist/overtype.min.js",
        name: "overtype.min.js",
        sha256: "92c9ed3de0492c9e3caf10e99e85b5d12a47c05ba1be1617c7680abb9bac46c7",
    },
    Asset {
        url: "https://raw.githubusercontent.com/panphora/overtype/22742e0ba1537aa6c9974befe3b6c7cbcd04c2a6/LICENSE",
        name: "LICENSE-overtype",
        sha256: "436eecee4003545420d99f861f6b80050746aac96997906527573a6224f27b9d",
    },
    Asset {
        url: "https://raw.githubusercontent.com/primer/octicons/cc4e12df6ff8292447ba9141eaa2a6f6e1c59a85/LICENSE",
        name: "LICENSE-octicons",
        sha256: "da259c8bd0de62713ccdcf88910aebca810644f98c2c912bad814fc79ea778df",
    },
];

fn verify(asset: &Asset, bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let actual = format!("{:x}", Sha256::digest(bytes));
    if actual != asset.sha256 {
        return Err(io::Error::other(format!(
            "SHA-256 mismatch for {}: expected {}, got {actual}",
            asset.name, asset.sha256
        ))
        .into());
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    let assets_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");

    for asset in ASSETS {
        println!("cargo:rerun-if-changed=assets/{}", asset.name);
    }

    let client = Client::builder().user_agent("gh-mdp").build()?;
    for asset in ASSETS {
        let path = assets_dir.join(asset.name);
        if path.exists() {
            match verify(asset, &fs::read(&path)?) {
                Ok(()) => continue,
                Err(error) => println!("cargo:warning=Refreshing {}: {error}", asset.name),
            }
        }

        println!("cargo:warning=Downloading {}", asset.name);
        let blob = client.get(asset.url).send()?.error_for_status()?.bytes()?;
        verify(asset, &blob)?;
        fs::write(path, blob)?;
    }
    Ok(())
}
