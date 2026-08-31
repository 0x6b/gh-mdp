use std::{error::Error, fmt::Write, fs, io, path::Path};

use reqwest::blocking::Client;
use sha2::{Digest, Sha256};

struct Asset {
    url: &'static str,
    name: &'static str,
    sha256: &'static str,
}

const ASSETS: &[Asset] = &[
    Asset {
        url: "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.12.0/highlight.min.js",
        name: "highlight.min.js",
        sha256: "8ab71eb09c51f501e5e25157d9cff100e46cc29bcbfc744d0b746d451fca7f53",
    },
    Asset {
        url: "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.12.0/styles/github.min.css",
        name: "highlight-github.min.css",
        sha256: "3a9a5def8b9c311e5ae43abde85c63133185eed4f0d9f67fea4b00a8308cf066",
    },
    Asset {
        url: "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.12.0/styles/github-dark.min.css",
        name: "highlight-github-dark.min.css",
        sha256: "9f208d022102b1d0c7aebfecd8e42ca7997d5de636649d2b31ea63093d809019",
    },
    Asset {
        url: "https://raw.githubusercontent.com/highlightjs/highlight.js/f7f7d3803bd898e37c017ffb881317f0cde04a70/LICENSE",
        name: "LICENSE-highlight.js",
        sha256: "6c081431591d9df696c82dc598fe1423765b8a299b200ed00b281afd0f64c490",
    },
    Asset {
        url: "https://cdn.jsdelivr.net/npm/morphdom@2.7.8/dist/morphdom-umd.min.js",
        name: "morphdom.min.js",
        sha256: "1bea1e3733860f75e851bf53e2cd97af8bd24d88e70f15cc643f4a737cd2fe2a",
    },
    Asset {
        url: "https://raw.githubusercontent.com/patrick-steele-idem/morphdom/8e004ca546b428dbe5a5e34a78b8af6179f85013/LICENSE",
        name: "LICENSE-morphdom",
        sha256: "dc18a39627c8f2e5391255635bd1d6bbb02e91ba4a9bd3e13e53347b8c25e61a",
    },
    Asset {
        url: "https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.9.0/github-markdown.min.css",
        name: "github-markdown.min.css",
        sha256: "3be3ba6f5b20f9e133688890012a1e20a0a6375efea59c214c424369d7694e3d",
    },
    Asset {
        url: "https://raw.githubusercontent.com/sindresorhus/github-markdown-css/265b22aa79815418195f453352456d5784bb7580/license",
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
    let mut actual = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(actual, "{byte:02x}")?;
    }
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
