# Katana

このリポジトリは、開発ドキュメントを Obsidian でも扱えるようにしている。

## Obsidian セットアップ

### 1. アプリを用意する

Obsidian が未導入なら、macOS では次で入れられる。

```bash
brew install --cask obsidian
```

### 2. Vault として開く

Obsidian で次のディレクトリを Vault として開く。

```bash
/Users/hiroyuki_furuno/works/private/katana
```

### 3. このリポジトリでの運用

- 共有設定は `.obsidian/` に置く
- 個人設定は `.gitignore` で除外する
- 作業メモは `docs/notes/` に置く
- 添付ファイルは `docs/assets/` に置く
- テンプレートは `docs/templates/` に置く
- 正式な仕様とタスクは `openspec/` を正本にする

## Obsidian と OpenSpec の役割

- Obsidian: 調査メモ、日次ログ、ドキュメント横断、作業中の整理
- OpenSpec: 提案、設計、タスクの正式な変更管理

設計判断や確定事項は `docs/` または `openspec/` に反映し、Obsidian のノートから参照する運用にする。

詳細は [docs/obsidian.md](./docs/obsidian.md) を参照。
