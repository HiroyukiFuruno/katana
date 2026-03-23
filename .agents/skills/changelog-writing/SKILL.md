---
name: changelog-writing
description: KatanAの変更をCHANGELOGに記録し、EN(UTC)とJA(JST)を同期管理するスキル。ユーザー目線の改善を優先し、内部変更はテンプレート化します。
---

# KatanA CHANGELOG Writing

`CHANGELOG.md` (EN) と `CHANGELOG.ja.md` (JA) を同期して管理するためのスキルです。

## 執筆原則

1. **同期管理**: EN版とJA版を必ず同時に更新し、構造（バージョン、セクション、項目数）を一致させます。
2. **ユーザー優先**: ユーザー目線の改善（パフォーマンス、UI変更、新機能）を優先的に、具体的に記載します。
3. **OpenSpec へのリンク (日本語限定)**: OpenSpec ワークフロー等のプロジェクト運用に関する変更を記載する場合、必ず実体ディレクトリ（例: [openspec/](./openspec/README.md)）への内部リンク（見出し付き推奨）を含めてください。ただし、**リンク先が日本語のみの場合は、日本語版 CHANGELOG (`CHANGELOG.ja.md`) にのみ記載し、英語版からは除外** してください。
4. **優先度表記の禁止**: `(P0)`, `(P1)`, `(P3 / P4)` などの **開発時優先度ラベルを CHANGELOG に記載してはいけません**。不具合の内容や影響範囲（「高負荷」「クラッシュ」等）を言葉で表現し、内部的な管理番号は含めないでください。
5. **内部変更の簡略化**: 内部の機能改善や運用上の変更は、後半のセクションにテンプレート的にまとめます。
6. **タイムスタンプ**:
    - **EN版**: `YYYY-MM-DD HH:MM:SS (UTC)` 形式
    - **JA版**: `YYYY-MM-DD HH:MM:SS (JST)` 形式
    - 過去のリリースについても、GitHub Releaseの時刻（UTC）を元に遡って更新します。

## セクション優先順位

1. `🚀 Features` / `🚀 新機能`
2. `🎨 UI/UX` / `🎨 UI/UX`
3. `⚡ Performance` / `⚡ パフォーマンス改善`
4. `🐛 Bug Fixes` / `🐛 バグ修正`
5. `🔧 Miscellaneous` / `🔧 その他` (内部改善、CI、依存関係更新等)

## テンプレート

### English (CHANGELOG.md)

```markdown
## [X.Y.Z] - YYYY-MM-DD HH:MM:SS (UTC)

### 🚀 Features
- Description of new user-facing functionality

### 🎨 UI/UX
- Description of visual or interaction improvements

### ⚡ Performance
- Description of speed or resource usage improvements

### 🐛 Bug Fixes
- Fix for [Issue/Description]

### 🔧 Miscellaneous
- Internal: [Description]
- Ops: [Description]
- Dependency updates
```

### Japanese (CHANGELOG.ja.md)

```markdown
## [X.Y.Z] - YYYY-MM-DD HH:MM:SS (JST)

### 🚀 新機能
- ユーザー向けの新しい機能の概要

### 🎨 UI/UX
- デザインやインタラクションの改善内容

### ⚡ パフォーマンス改善
- 速度向上やリソース消費の削減内容

### 🐛 バグ修正
- [内容/現象]の修正

### 🔧 その他
- 内部改善: [内容]
- 運用改善: [内容]
- 依存関係の更新
```
