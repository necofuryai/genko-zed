# Contributing to Genko

Genko accepts bug reports, feature requests, and pull requests in English or Japanese.

## Before starting

- Small bug fixes and documentation corrections may be submitted directly as pull requests.
- Open an issue before implementing a large behavior change, grammar change, new language mode, or public interface change.
- Report security vulnerabilities privately by following the [Security Policy](SECURITY.md) instead of opening a public issue.
- Use a minimal fictional manuscript when reproducing a problem. Do not include unpublished writing, personal information, credentials, or private logs.

## Development environment

The repository pins Rust in `rust-toolchain.toml`, Node.js in `.node-version`, and pnpm in `tree-sitter-genko/package.json`.
Install the declared Node.js version with your preferred version manager and use the declared pnpm version.
If you use `rustup`, install the components and target required by CI:

```bash
rustup component add rustfmt clippy
rustup target add wasm32-wasip2
```

Run the Rust checks from the repository root:

```bash
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --package genko-zed --lib
cargo test --locked --package genko-ls --all-features
cargo build --locked --release --package genko-zed --target wasm32-wasip2
```

Run the grammar checks from `tree-sitter-genko`:

```bash
pnpm install --frozen-lockfile
pnpm run generate
git diff --exit-code -- src
pnpm test
```

## Pull requests

- Target the `main` branch from a short-lived branch or fork.
- Keep each pull request focused on one reviewable change.
- Add or update tests for behavior changes.
- Regenerate the parser and include the generated files when changing `grammar.js`.
- Describe user-visible behavior and compatibility implications.
- List the validation commands that you ran.
- Resolve review conversations before merge.

Maintainers use squash merges.
GitHub Actions workflows from external contributors require maintainer approval before they run.
Maintainers also handle version changes, release tags, GitHub Releases, and Zed Extension Registry updates.

## License

By contributing, you agree that your contribution is provided under the repository's MIT License.
