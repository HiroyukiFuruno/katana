<p align="center">
  <img src="assets/icon.iconset/icon_128x128.png" width="128" alt="KatanA Desktop">
</p>

<h1 align="center">KatanA Desktop</h1>

<p align="center">
  macOS向けの高速・軽量なMarkdownワークスペース — Rustとeguiで構築。
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="https://github.com/HiroyukiFuruno/katana/actions/workflows/ci.yml"><img src="https://github.com/HiroyukiFuruno/katana/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/HiroyukiFuruno/katana/releases/latest"><img src="https://img.shields.io/github/v/release/HiroyukiFuruno/katana" alt="Latest Release"></a>
  <img src="https://img.shields.io/badge/platform-macOS-lightgrey" alt="Platform: macOS">
</p>

<p align="center">
  <a href="README.md">English</a> | 日本語
</p>

---

## KatanA とは

**KatanA** という名前は、日本語の **「刀（かたな）」** に由来しています。精密に鍛え上げられた、鋭利な刃物です。

開発における複雑な課題を、刀のように **鋭く、的確に切り崩していきたい** — そんな思いからこのプロジェクトは命名されました。問題をひとつずつ、切れ味よく解決していくためのツールでありたいという願いが込められています。

KatanA Desktop は、仕様書や技術ドキュメントを扱う開発者のために設計された、macOS向けの高速・軽量なMarkdownワークスペースです。

末尾の大文字 **A** は **「Agent」** を表しています。KatanA は AIエージェントと共に仕様駆動開発を行う時代のために設計されたツールであり、Markdownの仕様書が人間と AI の橋渡しとなる世界を見据えています。**Katana × Agent = KatanA.**

---

## 背景

**2026年**現在、ソフトウェア開発は **AIエージェント** の台頭により急速に進化しています。AIがコードの作成、レビュー、メンテナンスを支援する時代です。

この流れと並行して、**仕様駆動開発（SDD: Spec-Driven Development）** が注目されています。実装の前に仕様、アーキテクチャ記述、タスクを定義する開発手法であり、これらの仕様は通常 **Markdownドキュメント** として記述され、開発者とAIエージェント双方の共通基盤となります。

しかし、既存のMarkdownツールの多くは：

- 技術ドキュメントのワークフローに最適化されていない汎用エディタか、
- 不必要に複雑な重量級のナレッジ管理ツールのどちらかです。

**KatanA Desktopはこの問題を解決するために生まれました。**

KatanAの目標は、**シンプルで高速、ワークスペース指向のMarkdown環境** を提供し、開発者がSDDワークフローで使用するドキュメントを簡単に **閲覧・編集** できるようにすることです。

---

## 主な機能

- **ライブスプリットビュープレビュー** — 左側で編集、右側でレンダリング結果をスクロール同期しながら表示
- **ダイアグラムレンダリング** — Mermaid、PlantUML、Draw.ioのフェンスコードブロックをファーストクラスでサポート
- **GitHub Flavored Markdown** — テーブル、取り消し線、タスクリスト、脚注、自動リンク
- **ワークスペース対応** — フォルダを開き、統合ファイルツリーからファイルをナビゲーション
- **タブバー** — VSCodeスタイルのタブで複数ドキュメントを同時に開く
- **i18n（国際化）** — UI文字列は完全にローカライズ済み（英語・日本語同梱）
- **高速なネイティブパフォーマンス** — Rustとeguiでビルドされたネイティブバイナリ。Electron不要、Node.js不要

---

## インストール

> 現在は **macOSのみ** 対応。Apple SiliconとIntelの両方をサポートしています。

### ダウンロード

1. [Releasesページ](https://github.com/HiroyukiFuruno/katana/releases/latest) にアクセス
2. 最新の `KatanA-Desktop-x.y.z.dmg` をダウンロード
3. DMGを開き、**KatanA Desktop.app** を **アプリケーション** フォルダにドラッグ

### 初回起動時の注意

KatanA Desktop はアドホック署名済みですが、Appleの公証を受けていないため、初回起動時に **「未確認の開発元」** の警告が表示されます。

**オプションA: 右クリックで開く（macOSの設定に依存します）**

1. アプリケーションフォルダ内の **KatanA Desktop.app** を右クリック（またはControl+クリック）
2. コンテキストメニューから **「開く」** を選択
3. 確認ダイアログで **「開く」** をクリック

**オプションB: コマンドライン（推奨 / 確実な方法）**

```sh
xattr -cr /Applications/KatanA\ Desktop.app
```

初回の起動に成功すれば、以降はmacOSが選択を記憶し、通常通り起動できます。

---

## 現在のステータス

KatanA Desktop は **活発に開発中** です。最新バージョンと変更履歴は [Releasesページ](https://github.com/HiroyukiFuruno/katana/releases/latest) をご覧ください。

現在利用可能な主な機能：

- ワークスペースベースのMarkdown閲覧
- ダイアグラムサポート（Mermaid / PlantUML / draw.io）
- スクロール同期付きスプリットプレビュー
- 高速なネイティブデスクトップパフォーマンス（Rustベース）

プロジェクトは急速に進化しており、新機能や改善が頻繁にリリースされています。

---

## プロジェクトの目標

KatanAは開発者が以下を実現するためのツールを目指しています：

- Markdownドキュメントを効率的に閲覧・ナビゲーション
- 仕様駆動ワークフローとの連携
- 現代的なAIアシスト開発とのドキュメント統合

長期的なビジョンは、モダンな開発ツールを補完する **軽量なドキュメントワークスペース** の構築です。

---

## アイデアを募集しています

本プロジェクトはまだ初期段階にあります。

以下を歓迎します：

- 機能のアイデア
- ユーザビリティの提案
- バグレポート
- デザインに関するフィードバック
- 開発者からのコントリビューション

KatanAが開発者のドキュメントワークフローをどう改善できるか、アイデアがあれば [issue](https://github.com/HiroyukiFuruno/katana/issues) または [discussion](https://github.com/HiroyukiFuruno/katana/discussions) を開いてください。

皆さまのフィードバックがプロジェクトの方向性に直接影響します。

---

## オープンソースへのコミットメント

KatanA Desktopはオープンソースプロジェクトです。

**コアな機能を無料で提供し続ける** ことをお約束します。特に運用コストがかからない機能については：

- Markdownの閲覧
- ワークスペースナビゲーション
- ドキュメントブラウジング
- ダイアグラムレンダリング

---

## 今後の展望

一部の高度な機能は外部サービスや運用コストが必要になる可能性があります。

持続可能性のために、プロジェクトは以下を導入する可能性があります：

- オプションの有料機能（例: AIアシストツール）
- アプリケーション内の小規模な広告

ただし、**コアなドキュメント機能は引き続き無料** です。

---

## 開発者の方へ

ソースからビルドしたい方、コントリビュートしたい方、アーキテクチャを理解したい方へ：

- 📖 **[Development Guide](docs/development-guide.md)** — セットアップ、ビルド、テスト、プロジェクト構造
- 📖 **[開発ガイド（日本語）](docs/development-guide.ja.md)** — 日本語版の開発ガイド
- 📐 **[Coding Rules](docs/coding-rules.md)** — コードスタイル、規約、品質ゲート
- 🏗️ **[Architecture Decisions](docs/adr/)** — 設計の根拠とADR

---

## プロジェクトを支援する

KatanAが役に立ったと思っていただけたら、スポンサーシップで開発を支援できます。

<a href="https://github.com/sponsors/HiroyukiFuruno"><img src="https://img.shields.io/badge/Sponsor-❤️-ea4aaa?style=for-the-badge&logo=github-sponsors" alt="Sponsor"></a>

支援は以下に活用されます：

- 開発時間
- インフラ
- ツール費用

👉 **[スポンサーになる](https://github.com/sponsors/HiroyukiFuruno)**

- ⭐ このリポジトリにスターをつける — 他の人がKatanAを見つけるのに役立ちます
- KatanAを役立ててくれそうな人にシェアする

---

## ライセンス

KatanA Desktop は [MIT License](LICENSE) の下でリリースされています。
