# Genko

日本語 | [English](README.en.md)

Genko は、[Zed](https://zed.dev/) で日本語小説を書くための軽量な執筆支援拡張です。
初回版 [v0.1.0](https://github.com/necofuryai/genko-zed/releases/tag/v0.1.0) は、`.genko` 原稿の構文ハイライトと本文文字数の確認に絞っています。

## できること

- `.genko` ファイルを `Genko Novel` として自動認識する
- `#` 見出し、`「」` と `『』` の会話文、明示ルビ、カクヨム形式の傍点、青空文庫注記、HTML コメントをハイライトする
- 文書内の任意位置で `editor: hover` を実行すると、文書全体の本文文字数を表示する

```text
# 第一章

「｜原稿《げんこう》に《《傍点》》を付ける」
本文［＃ここから傍点］<!-- 編集メモ -->
```

## インストール

Genko は現在、[Zed Extension Registry への登録審査中](https://github.com/zed-industries/extensions/pull/7091)です。
Registry で公開された後は、Zed の Extension Gallery を開き、`Genko` を検索してインストールできます。

Zed Extension Registry で公開される前に試す場合は、開発版としてインストールしてください。
[Zed の開発版拡張](https://zed.dev/docs/extensions/developing-extensions)のビルドには Rust toolchain が必要です。
`rustup` を使わない場合は、`wasm32-wasip2` target を利用できるようにしてください。

```bash
git clone https://github.com/necofuryai/genko-zed.git
```

Zed で `zed: install dev extension` を実行し、`git clone` で取得した `genko-zed` ディレクトリを選択します。
`genko-ls` が `PATH` にない場合、拡張はキャッシュ済みバイナリを再利用するか、対応するバイナリを [GitHub Releases](https://github.com/necofuryai/genko-zed/releases) から自動で取得します。

## 使い方

1. `.genko` 拡張子の原稿を作成する。
2. Zed が言語を `Genko Novel` と認識したことを確認する。
3. 原稿内にカーソルを置き、Command Palette から `editor: hover` を実行する。

hover には `本文文字数: N 文字` と表示され、編集内容に追従して更新されます。

## 本文文字数の数え方

- Unicode の拡張書記素クラスタを一文字として数える
- 空白とタブは数え、改行は数えない
- 完結した明示ルビ `｜漢字《かんじ》` は、本文の `漢字` だけを数える
- 完結した青空文庫注記 `［＃…］` と HTML コメントは数えない
- 未完または不正な記法は、そのまま本文として数える
- 暗黙ルビ `漢字《かんじ》` と傍点 `《《本文》》` は、v0.1.0 では記号を含めて数える

上記以外の文字や記号は本文文字数に含まれます。

## v0.1.0 の範囲外

- `.txt` と `.md` の `Genko Novel` への割り当て
- アウトライン、折りたたみ、code action
- 原稿用紙換算、節単位の集計、settings
- tech モードと Zenn、Qiita 対応
- ステータスバー表示と縦書きプレビュー

## 対応プラットフォーム

v0.1.0 では、次の `genko-ls` リリースバイナリを提供しています。

- macOS：Apple Silicon、Intel
- Linux：ARM64、x86_64
- Windows：ARM64、x86_64

32-bit x86 は対象外です。

## 開発

CI は Rust 1.90、Node.js 24、pnpm 10.30.1 を使用します。

```bash
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --package genko-zed --lib
cargo test --locked --package genko-ls --all-features
cargo build --locked --release --package genko-zed --target wasm32-wasip2
```

`tree-sitter-genko` の grammar 生成とテストは、同ディレクトリで実行します。

```bash
pnpm install --frozen-lockfile
pnpm run generate
pnpm test
```

## コントリビューション

不具合報告、機能提案、pull request は日本語と英語で受け付けています。
再現例には架空の最小原稿を使い、未公開作品や個人情報を投稿しないでください。
大きな仕様変更を実装する前に、[GitHub Issues](https://github.com/necofuryai/genko-zed/issues) で提案してください。
開発環境と pull request の手順は [CONTRIBUTING.md](CONTRIBUTING.md) を参照してください。

脆弱性の詳細は公開 Issue に書かず、[Security Policy](SECURITY.md) に従って非公開で報告してください。

## ライセンス

[MIT License](https://github.com/necofuryai/genko-zed/blob/main/LICENSE)
