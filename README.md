# tarminal-session-gist-save

ターミナルのセッションログを GitHub Gist に保存する Rust 製 CLI です。

## インストール

```bash
cargo install --path .
```

## 使い方

`GITHUB_TOKEN` を設定して実行します（`gist` 権限が必要）。

```bash
export GITHUB_TOKEN=ghp_xxx
```

ファイルから保存（デフォルトは secret gist）:

```bash
tarminal-session-gist-save --file ./session.log
```

標準入力から保存:

```bash
cat ./session.log | tarminal-session-gist-save
```

public gist にする:

```bash
tarminal-session-gist-save --file ./session.log --public
```

追加オプション:

```bash
tarminal-session-gist-save \
  --file ./session.log \
  --name my-session.txt \
  --description "debug logs"
```

成功すると作成した Gist URL を標準出力に表示します。
