# gh-mdp

A GitHub Flavored Markdown live preview server that:

- Reloads changes automatically
- Preserves your scroll position as much as possible
- Offers GitHub-style rendering
- Serves relative links (images, files) from the markdown's directory
- Renders linked markdown files with the same template
- Integrates all resources into a single binary (no internet connection required)
- Can be used as a `gh` extension
- Runs quickly

## Installation

You can use the binary `gh-mdp` standalone.

```console
$ cargo install --git https://github.com/0x6b/gh-mdp
$ gh-mdp README.md
```

You can also install this as a [GitHub CLI](https://cli.github.com/) (`gh`) extension.

```console
$ gh extension install 0x6b/gh-mdp
```

## Usage

```console
$ gh mdp --help
A GitHub Flavored Markdown live preview server

Usage: gh-mdp [OPTIONS] [FILE]

Arguments:
  [FILE]  Markdown file or directory to preview (defaults to README.md if not
          specified, or looks for README.md in the given directory)

Options:
  -b, --bind <BIND>  Bind address [default: 127.0.0.1]
      --no-open      Don't open browser automatically
  -h, --help         Print help
  -V, --version      Print version
```

## License

MIT. See [LICENSE](./LICENSE) for details.

### Third-party assets

The third-party assets are downloaded at build time and embedded into the final product. Please review the respective licenses for more details.

| Asset                                                                      | License      | Source                                          |
| -------------------------------------------------------------------------- | ------------ | ----------------------------------------------- |
| [github-markdown-css](https://github.com/sindresorhus/github-markdown-css) | MIT          | [LICENSE](./assets/LICENSE-github-markdown-css) |
| [highlight.js](https://highlightjs.org/)                                   | BSD-3-Clause | [LICENSE](./assets/LICENSE-highlight.js)        |
| [mermaid](https://mermaid.js.org/)                                         | MIT          | [LICENSE](./assets/LICENSE-mermaid)             |
| [morphdom](https://github.com/patrick-steele-idem/morphdom)                | MIT          | [LICENSE](./assets/LICENSE-morphdom)            |

The favicon and header icon use the [Markdown mark](https://commons.wikimedia.org/wiki/File:Markdown-mark.svg) from Wikimedia Commons (CC0/Public Domain).
