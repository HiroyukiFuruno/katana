# Changelog

All notable changes to KatanA Desktop will be documented in this file.

## [0.2.0] - 2026-03-20

### 🐛 Bug Fixes

- Enforce strict lazy loading and restrict folder auto-expand on Open All
- Abolish redundant filename tooltip and fix ast linter coverage
- Restore missing absolute path in metadata tooltip and apply TDD

### 👷 CI/CD

- DMGのSHA256ハッシュ(checksums.txt)を自動生成・添付するよう改善

### 📚 Documentation

- CHANGELOG.ja.md に v0.1.6 のリリースノートを追加し Makefileのリリース対象に含める

### 🔧 Miscellaneous

- Refactor RwLock usage and fix external image caching on force reload

### 🚀 Features

- ワークスペース永続化・タブ復元ロジックの追加 (Task 1)
- Implement CacheFacade and stabilize all integration tests
- ワークスペースの再帰的展開・全て開くの実装と操作性改善 (Task 3, 5)
- Localize metadata tooltips and apply to file items

## [0.1.6] - 2026-03-19

### 🐛 Bug Fixes

- DMGビルド時にInfo.plistへ自動でバージョンを注入するよう改善
- Make release実行時にCargo.lockが自動同期されるよう改善

### 📚 Documentation

- プロジェクト固有スキル「release_workflow」を追加

### 🔧 Miscellaneous

- V0.1.7 リリース準備
- Cargo.lock の更新 (v0.1.7版への追従)
- V0.1.6 リリース準備

### 🚀 Features

- ワークスペースの検索とフィルター機能の実装
- 検索モーダルのInclude/Excludeオプション向け国際化テキストを追加
- 検索モーダルに包含/除外フィルター機能を追加し検索ボタンをUIに配置

### 🧪 Testing

- 検索フィルターのInclude/Excludeオプション機能に関する結合テストを追加

## [0.1.5] - 2026-03-19

### ♻️ Refactoring

- HashMapと固定長配列をVecへ統一し、ASTルールやマイグレーション機能を含めて一括適用

### 🔧 Miscellaneous

- Bump version to 0.1.4

### 🚀 Features

- Apply v0.1.5 changes and bump version to 0.1.5

### 🧪 Testing

- Fix tests broken by workspace methods renaming
- Add missing tests to meet 100% coverage gate

## [0.1.4] - 2026-03-19

### 🧪 Testing

- リポジトリ肥大化・CI失敗の原因となっていたUI画像スナップショットテストを完全に廃止し、セマンティックなアサーションへと移行

## [0.1.3] - 2026-03-19

### ♻️ Refactoring

- Settings.jsonの構造を階層化（ThemeSettings/FontSettings/LayoutSettings）しマイグレーション機構を追加
- カバレッジゲート修正とコード品質改善

### 🐛 Bug Fixes

- V0.1.3 バージョン更新漏れのリカバリー
- 並行テスト実行時の環境変数汚染による curl 起動失敗の flaky なテストを修正

### 📚 Documentation

- Markdownlint設定を.vscode管理に移行し、v0.1.3リリースノートを追加

### 🔧 Miscellaneous

- Make checkからsnapshotテストと冗長な統合テスト実行を除外

### 🚀 Features

- テーマプリセットを10→30種に拡充（OneDark/TokyoNight/CatppuccinMochaなど追加）
- I18nを型安全な構造体（I18nMessages）へ移行し、8言語（zh-CN/zh-TW/ko/pt/fr/de/es/it）を追加
- MacOSネイティブメニューに8言語のタグを追加し、言語切替に合わせてメニュー文字列を動的翻訳
- UI全体をi18n/設定階層化に対応させ、設定画面にOS言語検出・テーマ拡充・Show more/lessトグルを実装

### 🧪 Testing

- I18n型安全化・設定階層化・テーマ拡充に合わせてテストを更新（integration/i18n/theme/diagram_renderingテスト）

## [0.1.2] - 2026-03-19

### 🐛 Bug Fixes

- ワークスペースファイルエントリの左寄せ修正
- ライトテーマでフォントサイズスライダーが不可視になる問題を修正
- スライダーにselection colorのborderを付与し全テーマで視認性確保
- Markdownプレビューのテーブルが利用可能幅を使うよう修正
- テーブルレイアウトおよび上下分割スクロールの不具合を修正

### 📚 Documentation

- READMEにダイアグラム表示ガイドとbrewアップデート方法を追加
- Snapshot禁止(NG)ルールをcoding-rules・self-reviewに追記
- Brewアップデート方法をREADMEに追加

### 🔧 Miscellaneous

- V0.1.2 リリース準備
- Fix flaky view mode integration test by adding ui stabilization steps
- Warningのerror化と未使用コードの削除
- V0.1.2 リリース準備

### 🚀 Features

- タブナビ・スライダーにi18nツールチップ追加

### 🧪 Testing

- UIバグに対するTDD検証テストの追加とスナップショット更新

## [0.1.1] - 2026-03-19

### 🐛 Bug Fixes

- Homebrew Cask更新ステップにエラーハンドリングを追加
- キャッシュされた古いDMGファイルの混入を防止
- Emoji.rsのmacOS専用コードにcfgガードを追加（Linux CI Lint修正）

### 🚀 Features

- 隠しディレクトリのワークスペースツリー表示対応とディレクトリ更新ボタンの追加

## [0.1.0] - 2026-03-19

### ⏪ Reverted

- Release.ymlとREADMEをv0.0.3の状態に戻す
- V0.0.4の変更を取り消し、v0.0.3の状態に戻す

### ♻️ Refactoring

- Make ci → make check リネーム + make check-light 新設
- Os_fonts.rs のインラインテストを tests/ に移動し日本語コメントを英語化

### 🎨 Styling

- #[allow] 属性に理由コメントを追加（coding-rules セクション10準拠）

### 🐛 Bug Fixes

- .app署名を改善（--deep廃止、runtime/timestamp指定、DMGは未署名のまま）
- 起動時にワークスペースが復元されないデグレードを修正
- Vendor egui_commonmarkに絵文字対応パッチを適用

### 📚 Documentation

- Define Versioning Policy and refine CI triggers
- Make check コマンドのインデントを他コマンドと統一
- コーディング規約および自己レビュー基準の更新
- コードブロックの言語指定（text）を日英版で統一
- CHANGELOG v0.1.0-dev 追記およびREADME.ja.mdの同期修正

### 🔧 Miscellaneous

- Pre-push hookにglobフィルタを追加し、コード変更を伴わないpushのCI実行をスキップ
- V0.0.4 リリース準備
- Make check-light から fixture テストを除外
- スナップショット更新時の不要な旧バックアップ画像 (.old.png) をGit追跡対象から除外
- カバレッジゲートの除外ルールを拡張（return None/false/display/Pending）
- V0.1.0 リリース準備（バージョン番号更新）

### 🚀 Features

- Homebrew Cask対応を追加
- テーマプリセット10種と ThemeColors 基盤を実装（Task 1） (#23)
- フォントサイズ・ファミリー設定の基盤実装（Task 2）
- テーマ連動・設定画面実装とスナップショットの更新 (WIP)
- OSフォントの動的取得とUIへの反映機能を追加
- タスク4 エディタ/プレビューレイアウト設定を実装
- タスク5 OSテーマ連動（初回デフォルト自動選択）を実装
- タスク6 フォント設定拡張（検索機能 + Apple Color Emoji）を実装
- Linterに厳密な品質チェック（todo!マクロ等の使用禁止など）を追加
- フォント検索・絵文字対応およびプレビュー等のUI機能改善
- 絵文字インライン描画基盤の実装とSVG/HTTPキャッシュローダーの分離
- AST Linterにlazyコード検出テストと#[cfg(test)]モジュール除外を追加

### 🧪 Testing

- カバレッジ改善のためのテスト追加

## [0.0.3] - 2026-03-18

### ♻️ Refactoring

- マジックナンバー定数化とAST linterテスト拡充
- Ignoreタグをlimited_localに統一

### 🐛 Bug Fixes

- Coverageジョブをローカルのmake coverageと統一
- ダークテーマでのDrawIo図テキスト視認性を改善
- .appバンドルからのmmdc解決を6層フォールバックに拡張
- ダイアグラム系スナップショットテストをCI環境でスキップ
- Mmdc実行時にnodeのPATHを補完してGUIアプリからの起動を修正
- HTMLブロックの上下に余白を追加しレイアウトの窮屈さを解消

### 📚 Documentation

- READMEのバージョン固定セクションを動的なステータス表記に変更

### 🔧 Miscellaneous

- カバレッジ除外理由を正確な技術的根拠に更新
- V0.0.3 リリース準備
- リリースノートをCHANGELOG.mdからの抽出に変更

### 🧪 Testing

- スナップショットテストのCI環境依存エラーを修正
- I18nの複数テストにおけるグローバルステートの競合エラーを修正
- ダイアグラムレンダリングとサンプルフィクスチャの統合テストを追加

## [0.0.2] - 2026-03-17

### 🐛 Bug Fixes

- Resolve linux cross-compilation errors for github actions
- Resolve markdown rendering, i18n label update, and CI coverage flakiness
- Support CenteredMarkdown for raw HTML alignment reproduction
- CenteredMarkdown の中央寄せ・画像パス解決・バッジ表示を修正

### 📚 Documentation

- 初開起動時のxattrコマンド手順を復元

### 🔧 Miscellaneous

- Kick ci to retry integration tests
- Release v0.0.2
- Include Cargo.lock and CHANGELOG.ja.md in release v0.0.2

### 🧪 Testing

- Update integration test snapshot
- Increase snapshot tolerance to 4000 to absorb CI/local macOS text rendering differences

## [0.0.1] - 2026-03-16

### ⏪ Reverted

- Sccacheを撤回、キャッシュパス最適化のみ維持

### ♻️ Refactoring

- Drawio_renderer のclippy警告を修正
- テストを src/ から tests/ ディレクトリに移行し、Clippy を厳格化
- Katana-ui を lib/binary 構造にリファクタリングし、ロジックを抽出
- マジックナンバーを用途明確な名前付き定数に抽出
- 言語定義をlocales/languages.jsonに外部化
- Span_locationの重複をフリー関数に統合(自己レビュー修正)
- Egui描画ロジックとイベントルーティングの分離
- ソースコードとテストの日本語コメント・文字列を英語化
- UIレイアウト改善とリンターモジュール追加

### ⚡ Performance

- CI/CDにsccacheとキャッシュ最適化を導入

### 🐛 Bug Fixes

- Clippy 警告・フォーマット・30行制限の修正
- スクリーンショットで確認した問題を修正
- PLANTUML_JAR を排他的オーバーライドにしてテストを安定化
- 3問題を修正 — レイジーロード・Mermaidフォント・デスクトップ強制移動
- スナップショットテストのフレーキー問題を修正
- Eguiのレイアウト制約を回避するためリスト内のコードブロックを前処理でデインデントする
- MacOS sed互換性のためInfo.plist更新をPerlに変更
- Release CDにad-hocコード署名を追加
- CIトリガーのブランチ名をmasterに修正、Cargo.lock更新
- Sccache-actionのSHA修正、CHANGELOG英語/日本語版を整備
- Cargo installでsccacheを一時無効化（競合回避）
- CI LintジョブでRUSTC_WRAPPERを無効化（clippy互換性）

### 📚 Documentation

- Mark test compilation, tab UI, and plantuml macos bug as done
- Coding-rulesにi18n規約（セクション11）を追加
- README・ドキュメントテンプレート追加、.obsidianをgitignore対象に変更
- プロジェクト基盤ファイル追加 — LICENSE(MIT)、README、開発環境セットアップスクリプト
- ADR(Architecture Decision Records)と統合テストシナリオを追加
- 技術的負債メモ(TECHNICAL_DEBT.md)を追加
- Organize-documentsのopenspecを追加
- 共通ドキュメント周りの英語化と日本語版(*.ja.md)の並行整備およびopenspecアーカイブ
- プロジェクト名をKatanAに統一（README、Cargo.toml、設定コメント）
- ドキュメントを一般配布向けに再構築 (#21)
- 「What is KatanA」セクションを追加（英語/日本語）
- KatanAの末尾「A」= Agentの由来を追記

### 🔧 Miscellaneous

- Bootstrap katana repository
- Remove opsx prompt files
- Align gitignore with official templates
- Task 6.2 完了マーク — bootstrap-katana-macos-mvp 全タスク完了
- Openspecディレクトリをgit管理から除外
- Gitignore更新（openspec, obsidian設定, katana-core .gitignore統合）
- 不要なドキュメントテンプレートとREADMEを削除
- CI カバレッジジョブ追加と品質ゲートの明文化
- Desktop-viewer-polishに向けたCI要件の厳格化と不要アセット削除
- Lefthookの検証コマンドをMakefileに統合・自動修正化
- 依存関係の更新 (dirs-sys 0.5.0, rfd 0.17.2, egui_commonmark features追加)
- GitHub Sponsors用のFUNDING.ymlを追加
- Cliff.tomlからCI botコミットを除外する設定を追加
- V0.0.1 リリース準備

### 🚀 Features

- Bootstrap Katana macOS MVP — Rust プロジェクト基盤と全コアモジュールの実装
- Task 3.2 — ネイティブ Markdown プレビューペイン実装
- I18n support, language setting, appAction expansion, bin rename
- ダイアグラムレンダリング改善（drawio矢印対応、mermaid PNG出力、CommandNotFound区別）
- ファイルシステムサービス拡張（ワークスペースツリー・ファイル監視改善）
- タブ別プレビュー管理、スクロール同期、macOSネイティブメニュー、ワークスペースパネル制御
- 検証強化 — lefthook導入、テスト追加、Clippy厳格化、品質ゲート定義
- AST Linter(katana-linter)を導入 — i18nハードコード文字列・マジックナンバー検知
- Apply Katana app icon and version for native About panel (#15)
- 設定の永続化基盤を実装（JsonFileRepository + SettingsService）
- ワークスペース・言語変更時に設定を自動保存
- 起動時に保存済み設定（ワークスペース・言語）を復元
- プレビュー機能改善 (画像パス解決、セクション分割の先頭フェンス対応、ダイアグラムレンダラー改善)
- About画面の改善とアプリ表示名KatanAへの統一
- MacOSアプリバンドル(.app)パッケージングの追加 (#18)
- MacOS DMGインストーラー生成の追加 (#19)
- リリース自動化（git-cliff + make release） (#20)
- リリースCDワークフロー(.github/workflows/release.yml)を新設 (#22)
- GitHub SponsorsのURL設定とREADME日本語版の追加

### 🧪 Testing

- Task 6.2 — プレビュー同期テスト追加
- Add app state unit tests and fix java headless mode for plantuml
- プレビュー同期のユニットテストを追加（タスク3.2完了）
- カバレッジ厳格化 — ignore-filename-regex 撤去・#[coverage(off)] 全廃・Regions 100% 強制
- LLVMカバレッジ算出の差異対応とテスト100%ゲートの厳密化
- 永続化ラウンドトリップの統合テストを追加


