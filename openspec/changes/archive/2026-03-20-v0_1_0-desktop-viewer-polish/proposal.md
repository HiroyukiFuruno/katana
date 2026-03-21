## Why

Katana Desktop は現在 Dark テーマ固定で、フォントサイズ・ファミリーの変更もできず、画面分割も固定です。Viewer アプリとして最も基本的なカスタマイズ機能が欠落しており、ユーザー体験の根幹に関わる。テーマ・フォント・レイアウト・設定UIの4つをまとめて実装し、v0.1.0 で「自分好みに使える」状態を実現する。

## What Changes

### テーマ設定（Theme Presets & Custom Base）
- `ThemeColors` struct に全UI色を集約
- **有名テーマプリセット計10種**（Dark系: KatanA-Dark, Dracula, GitHub Dark, Nord, Monokai / Light系: KatanA-Light, GitHub Light, Solarized Light, Ayu Light, Gruvbox Light）のパレット定義
- プリセットをベースとしてユーザーが色をカスタマイズできるデータ構造の設計と適用
- `AppState` への状態追加と `egui::Visuals` への変換
- ハードコーディングされた色定数を `ThemeColors` 参照に置き換え
- テーマ切替の即時反映・永続化

### フォント設定
- フォントサイズ（デフォルト14px、8–32px）とフォントファミリーのカスタマイズ
- エディタ・プレビューへのリアルタイム反映
- 永続化と復元

### 画面分割レイアウト設定（Split Layout）
- プレビュー上部の「Preview」等の不要なラベル削除
- ウィンドウリサイズに追従する完全50:50の等分レイアウト
- 左右分割（Horizontal）と上下分割（Vertical）の動的切り替え
- エディタとプレビューの配置順序の入れ替え（左/右、上/下）と状態の永続化

### 設定UI（独立ウィンドウ）
- アプリウィンドウ内に独立した設定ダイアログ（`egui::Window`等による別ウィンドウ/モーダル型UI）として実装
- 将来的な拡張性を持たせるため、**タブ切り替え（例: Theme, Font, Layout）構造**を基本とする
- テーマ選択、フォントスライダー、目次デフォルト表示、画面レイアウト設定を配置
- 設定変更時の自動保存

## Capabilities

### New Capabilities
- `theme-settings`: テーマ設定（Dark/Light切替）
- `font-settings`: フォントサイズ・フォントファミリー設定
- `layout-settings`: 画面分割レイアウト設定（縦横分割、均等配置、順序入替）

### Modified Capabilities
- `settings-persistence`: 設定UIパネルから直接設定を操作・保存

## Impact

- **katana-ui** (`app_state.rs`): テーマ・フォント設定の状態管理追加
- **katana-ui** (`shell.rs`): 設定パネルUI追加、テーマ色の一括適用
- **katana-ui** (`preview_pane.rs`): フォント設定連動
- **katana-platform** (`settings.rs`): テーマ・フォント設定の永続化
