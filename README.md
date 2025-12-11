# gh-mdp

A GitHub Flavored Markdown live preview server that:

- Watches all markdown files in the directory tree (respects `.gitignore`)
- Reloads changes automatically with DOM diffing (preserves scroll position)
- Offers GitHub-style rendering with syntax highlighting and Mermaid diagrams
- Serves relative links (images, files) from the markdown's directory
- Renders linked markdown files with the same template
- Integrates all resources into a single binary (no internet connection required)
- Can be used as a `gh` extension

## Installation

You can use the binary `gh-mdp` standalone.

```console
$ cargo install --git https://github.com/0x6b/gh-mdp
$ gh-mdp README.md
```

You can also install this as a [GitHub CLI](https://cli.github.com/) (`gh`) extension.

```console
$ gh extension install 0x6b/gh-mdp

# if you have already installed, upgrade to the latest
$ gh extension upgrade 0x6b/gh-mdp
```

## Usage

```console
$ gh mdp --help
A GitHub Flavored Markdown live preview server

Usage: gh-mdp [OPTIONS] [FILE]

Arguments:
  [FILE]  Markdown file or directory to preview (defaults to index.md or README.md)

Options:
  -b, --bind <BIND>  Bind address [default: 127.0.0.1]
      --no-open      Don't open browser automatically
  -h, --help         Print help
  -V, --version      Print version
```

When a directory is specified, it looks for `index.md` first, then `README.md`.

## License

MIT. See [LICENSE](./LICENSE) for details.

### Third-party assets

The third-party assets are downloaded at build time and embedded into the final product. Please review the respective licenses for more details.

| Asset                                                                      | License      | Source                                                                           |
| -------------------------------------------------------------------------- | ------------ | -------------------------------------------------------------------------------- |
| [github-markdown-css](https://github.com/sindresorhus/github-markdown-css) | MIT          | [LICENSE](https://github.com/sindresorhus/github-markdown-css/blob/main/license) |
| [highlight.js](https://highlightjs.org/)                                   | BSD-3-Clause | [LICENSE](https://github.com/highlightjs/highlight.js/blob/main/LICENSE)         |
| [mermaid](https://mermaid.js.org/)                                         | MIT          | [LICENSE](https://github.com/mermaid-js/mermaid/blob/develop/LICENSE)            |
| [morphdom](https://github.com/patrick-steele-idem/morphdom)                | MIT          | [LICENSE](https://github.com/patrick-steele-idem/morphdom/blob/master/LICENSE)   |

The favicon and header icon use the [Markdown mark](https://commons.wikimedia.org/wiki/File:Markdown-mark.svg) from Wikimedia Commons (CC0/Public Domain).

UI icons (copy, list, screen-full) are from [Octicons](https://primer.style/foundations/icons) (MIT).
