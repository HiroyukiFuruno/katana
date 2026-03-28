---
name: changelog-writing
description: KatanAの変更をCHANGELOGに記録し、EN(UTC)とJA(JST)を同期管理するスキル。ユーザー目線の改善を優先し、内部変更はテンプレート化します。
---

# KatanA CHANGELOG Writing

`CHANGELOG.md` (EN) と `CHANGELOG.ja.md` (JA) を同期して管理するためのスキルです。

## 執筆原則

1. **同期管理**: EN版とJA版を必ず同時に更新し、構造（バージョン、セクション、項目数）を一致させます。
2. **ユーザー目線の徹底**: 本CHANGELOGはアプリを利用するエンドユーザー向けです。システムの内部的な変数名（例: `ColorSettingDef`、`ThemeColors`、`egui::ComboBox`、関数名、クレート名等の実装詳細）は**絶対に記載しないでください**。必ず「ユーザーにとってどう便利になったか」を説明する客観的な表現へリライトしてください。
3. **OpenSpec へのリンク (日本語限定)**: OpenSpec ワークフロー等のプロジェクト運用に関する変更を記載する場合、必ず実体ディレクトリ（例: [openspec/](./openspec/README.md)）への内部リンク（見出し付き推奨）を含めてください。ただし、**リンク先が日本語のみの場合は、日本語版 CHANGELOG (`CHANGELOG.ja.md`) にのみ記載し、英語版からは除外** してください。
4. **優先度表記の禁止**: `(P0)`, `(P1)`, `(P3 / P4)` などの **開発時優先度ラベルを CHANGELOG に記載してはいけません**。不具合の内容や影響範囲（「高負荷」「クラッシュ」等）を言葉で表現し、内部的な管理番号は含めないでください。
5. **内部変更の簡略化**: リファクタリングや運用上の変更であっても、コードレベルの実装詳細は書かず、ユーザー目線での「結果（安定性や保守性の向上など）」として、後半のセクションにテンプレート的にまとめます。
6. **タイムスタンプの必須記載**: 日付情報の抜け漏れは厳禁です。必ず `git log --tags` 等を用いて対象のバージョンのタグ（またはリリース時点のコミット）の時刻を調べ、正確に取得してください。
    - **EN版**: `YYYY-MM-DD HH:MM:SS (UTC)` 形式
    - **JA版**: `YYYY-MM-DD HH:MM:SS (JST)` 形式
    - 新規に追記する際はもちろん、過去リリースに時刻が欠落している場合も、GitHub Release や git log の時刻を元に遡って更新してください。

## セクション優先順位

1. `🚀 Features` / `🚀 新機能`
2. `✨ Improvements` / `✨ 改善`
3. `🐛 Bug Fixes` / `🐛 不具合修正`
4. `🔧 System` / `🔧 その他` (リファクタリングなどは「パフォーマンス改善」「安定性向上」等と抽象的に記載する。CI/CDやOpenSpecなど、アプリ利用者に直接関係のない事柄は記載しないこと)

## テンプレート

### English (CHANGELOG.md)

```markdown
## [X.Y.Z] - YYYY-MM-DD HH:MM:SS (UTC)

### 🚀 Features
- Description of new user-facing functionality

### ✨ Improvements
- Description of visual, performance, or functionality enhancements

### 🐛 Bug Fixes
- Fix for [Issue/Description]

### 🔧 System
- Abstract summary of internal improvements or refactorings (e.g., "Improved background stability"). Do not document CI/CD, OpenSpec, or other non-app-facing technical details.
```

### Japanese (CHANGELOG.ja.md)

```markdown
## [X.Y.Z] - YYYY-MM-DD HH:MM:SS (JST)

### 🚀 新機能
- ユーザー向けの新しい機能の概要

### ✨ 改善
- 既存のUIや機能の改善・修正内容

### 🐛 不具合修正
- [内容/現象]の解消

### 🔧 その他
- 詳細な実装やリファクタリングは「処理効率の改善」や「内部安定性の向上」など抽象的な文言に留める。
- CI/CDパイプラインやOpenSpecなど、アプリの利用自体に関係ない運用上の事項は記載しない。
```
