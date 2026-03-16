# 変更履歴

KatanA Desktop の主な変更点をこのファイルに記録します。

## [0.0.1] - 2026-03-16

KatanA Desktop の最初のパブリックリリース 🎉

### 🚀 新機能

- Rust + egui によるネイティブ macOS Markdown ワークスペース
- スクロール同期付きライブスプリットビュープレビュー
- ダイアグラムレンダリング（Mermaid、PlantUML、Draw.io）
- GitHub Flavored Markdown サポート（テーブル、取り消し線、タスクリスト、脚注）
- ワークスペース対応のファイルツリーナビゲーション
- 複数ドキュメント対応のタブバー
- 国際化対応（英語・日本語）
- 設定の永続化（ワークスペース、言語）
- macOS .app バンドルと .dmg インストーラー
- チェンジログ自動生成付きリリースパイプライン
- スムーズなインストールのための Ad-hoc コード署名
- コーディング規約遵守のための AST リンター

### ♻️ リファクタリング

- egui 描画ロジックとイベントルーティングの分離
- マジックナンバーを名前付き定数に抽出
- テストを src/ から tests/ ディレクトリに移行
- 言語定義を locales/languages.json に外部化
- ソースコードのコメント・文字列を英語化

### 🐛 バグ修正

- リスト内のコードブロック表示を修正（前処理デインデント）
- スナップショットテストの不安定性を修正
- レイジーロード・Mermaidフォント・デスクトップ強制移動の修正
- macOS sed 互換性のための Info.plist 更新方法変更

### 📚 ドキュメント

- インストールガイド付きユーザー向け README（英語/日本語）
- 「KatanA とは」セクション — 名前の由来（Katana × Agent）
- 開発者ガイドを docs/ に移動
- ADR（Architecture Decision Records）
- コーディング規約と i18n 規約
- GitHub Sponsors 連携

### 🧪 テスト

- 100% ラインカバレッジゲート（cargo-llvm-cov）
- プレビュー同期の統合テスト
- 設定の永続化ラウンドトリップテスト
- CodeQL セキュリティスキャン

### 🔧 その他

- CI/CD パイプライン（GitHub Actions）と sccache 最適化
- git-cliff によるリリース自動化
- lefthook プリコミット/プリプッシュフック
- GitHub Sponsors 用 FUNDING.yml
