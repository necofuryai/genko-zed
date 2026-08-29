# Genko

[日本語](README.md) | English

Genko is a lightweight [Zed](https://zed.dev/) extension for writing Japanese fiction.
The initial [v0.1.0](https://github.com/necofuryai/genko-zed/releases/tag/v0.1.0) release deliberately focuses on syntax highlighting for `.genko` manuscripts and a document-wide body character count.

## Features

- Recognizes `.genko` files as `Genko Novel`
- Highlights `#` headings, dialogue in `「」` and `『』`, explicit ruby, Kakuyomu-style emphasis, Aozora Bunko annotations, and HTML comments
- Shows the document-wide body character count when you run `editor: hover` at any position

```text
# Chapter One

「｜原稿《げんこう》に《《傍点》》を付ける」
本文［＃ここから傍点］<!-- Editing note -->
```

## Installation

Genko is currently [under review for the Zed Extension Registry](https://github.com/zed-industries/extensions/pull/7091).
After it is published, open Zed's Extension Gallery, search for `Genko`, and install it.

Until then, install Genko as a dev extension.
Building a [Zed dev extension](https://zed.dev/docs/extensions/developing-extensions) requires a Rust toolchain.
If you do not use `rustup`, make sure the `wasm32-wasip2` target is available.

```bash
git clone https://github.com/necofuryai/genko-zed.git
```

Run `zed: install dev extension` in Zed and select the cloned `genko-zed` directory.
If `genko-ls` is not on `PATH`, the extension reuses a cached binary or automatically downloads the matching one from [GitHub Releases](https://github.com/necofuryai/genko-zed/releases).

## Usage

1. Create or open a manuscript with the `.genko` extension.
2. Confirm that Zed recognizes the language as `Genko Novel`.
3. Place the cursor anywhere in the manuscript and run `editor: hover` from the Command Palette.

The hover displays `Body character count: N` and updates as the document changes.

## Character-count semantics

- Counts Unicode extended grapheme clusters as characters
- Includes spaces and tabs, but excludes line breaks
- Counts only the base text in complete explicit ruby such as `｜漢字《かんじ》`
- Excludes complete Aozora Bunko annotations such as `［＃…］` and complete HTML comments
- Counts incomplete or malformed markup literally
- Counts implicit ruby such as `漢字《かんじ》` and Kakuyomu-style emphasis such as `《《本文》》` literally, including their delimiters, in v0.1.0

All other characters and punctuation are included in the body character count.

## Not included in v0.1.0

- Assigning `.txt` or `.md` files to `Genko Novel`
- Outline, folding, and code actions
- Manuscript-sheet conversion, section-level statistics, and settings
- Tech mode or Zenn and Qiita support
- Status-bar output or vertical-writing preview

## Supported platforms

v0.1.0 provides `genko-ls` release binaries for:

- macOS on Apple Silicon and Intel
- Linux on ARM64 and x86_64
- Windows on ARM64 and x86_64

32-bit x86 is not supported.

## Development

CI uses Rust 1.90, Node.js 24, and pnpm 10.30.1.

```bash
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --package genko-zed --lib
cargo test --locked --package genko-ls --all-features
cargo build --locked --release --package genko-zed --target wasm32-wasip2
```

Generate and test the grammar from the `tree-sitter-genko` directory.

```bash
pnpm install --frozen-lockfile
pnpm run generate
pnpm test
```

## Contributing

Bug reports, feature requests, and pull requests are welcome in English or Japanese.
Use a minimal fictional manuscript when reporting a problem, and do not post unpublished writing or personal information.
Open a [GitHub issue](https://github.com/necofuryai/genko-zed/issues) before implementing a large behavior change.
See [CONTRIBUTING.md](CONTRIBUTING.md) for the development environment and pull request process.

Do not disclose vulnerability details in a public issue.
Follow the [Security Policy](SECURITY.md) to report them privately.

## License

[MIT License](https://github.com/necofuryai/genko-zed/blob/main/LICENSE)
