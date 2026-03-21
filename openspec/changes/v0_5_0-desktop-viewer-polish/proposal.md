## Why

Markdown を他の形式（PDF/HTML/画像）でエクスポートする機能がなく、「共有」が不便。また、OSS として公開するにあたり、利用規約の同意フローが必要。エクスポート機能と初回起動体験を整備し、v0.5.0 で「配布・共有に耐える」状態を実現する。

## What Changes

### Markdown エクスポート
- メニューに「エクスポート」サブメニュー（HTML / PDF / PNG / JPG）
- HTML: comrak HTML出力 + CSS埋め込み → ブラウザで開く
- PDF/画像: 外部ツール（wkhtmltopdf/weasyprint）呼び出し
- 外部ツール未インストール時のエラーメッセージとガイド表示

### 規約同意画面
- `AppSettings` に規約同意フラグ（`terms_accepted_version`）追加
- 規約テキストの i18n（EN/JA）
- 全画面モーダルの規約同意UI（言語選択プルダウン付き）
- 同意 → 永続化、不同意 → アプリ終了
- 規約バージョン更新時の再同意要求
- 起動フロー（スプラッシュ → 規約 → メインUI）の統合

## Capabilities

### New Capabilities
- `markdown-export`: Markdown 変換・エクスポート（PDF/PNG/JPG/HTML）
- `terms-agreement`: 初回起動時の規約同意画面

## Impact

- **katana-platform** (`filesystem.rs`): エクスポート機能
- **katana-ui** (`shell.rs`): エクスポートメニュー、規約同意画面
- **katana-ui** (`app_state.rs`): 規約同意状態管理
- **Cargo.toml**: PDF出力ライブラリ等の依存追加
- **i18n**: 規約テキストリソース（EN/JA）追加
