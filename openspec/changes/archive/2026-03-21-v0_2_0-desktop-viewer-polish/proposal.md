## Why

アプリを再起動するたびにワークスペースディレクトリを毎回選び直す必要があり、日常的な利用に大きなストレスがある。また、大きなディレクトリを開いた際にUIがフリーズする問題もある。ワークスペースの永続化・非同期読み込み・コンテキストメニューを一括で実装し、v0.2.0 で「ストレスなくファイルを管理できる」状態を実現する。

## What Changes

### ワークスペース永続化
- `AppSettings` にワークスペースパスリスト（`Vec<String>`）を追加
- 重複チェック、自動保存、起動時の自動復元
- ワークスペース除去機能（UIボタン + 永続化削除）

### ワークスペースローディング表示
- ツリー構築の非同期化（`std::thread::spawn` + `mpsc::channel`）
- スピナー + 「読み込み中...」メッセージ表示
- UI全体の応答性維持（読み込み中もタブ操作可能）

### ワークスペースコンテキストメニュー
- フォルダ右クリック: 「全て開く」（直下Markdownのみ、重複防止）
- ファイル/ディレクトリ右クリック: メタ情報表示（サイズ、更新日時、パス等）

## Capabilities

### New Capabilities
- `workspace-persistence`: ワークスペースディレクトリの永続化
- `workspace-loading`: ワークスペース読み込み時のローディング表示
- `workspace-context-menu`: ツリーのコンテキストメニュー

### Modified Capabilities
- `workspace-shell`: ワークスペースパネルに永続化・非同期・コンテキストメニューを追加

## Impact

- **katana-core** (`workspace.rs`): 複数ワークスペース、永続化対応
- **katana-ui** (`shell.rs`): コンテキストメニュー、ローディングUI
- **katana-ui** (`app_state.rs`): `WorkspaceLoadState` enum 追加
- **katana-platform** (`settings.rs`): ワークスペースパスリストの永続化
