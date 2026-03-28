## MODIFIED Requirements

### Requirement: The shell layout preserves the MVP navigation model

The system SHALL present a desktop shell with dedicated areas for a workspace utility rail, workspace navigation, document editing, and preview rendering.

#### Scenario: Show the default MVP layout

- **WHEN** the application starts with an active workspace
- **THEN** the user sees a workspace utility rail, a workspace pane, an editor pane, and a preview pane
- **THEN** the workspace utility rail remains available even when the workspace pane is collapsed
- **THEN** the shell reserves a consistent location for future menu and AI panel expansion

## ADDED Requirements

### Requirement: ワークスペースペインはタイトル文字列なしで表示される

ワークスペースペインのヘッダーは、`Workspace` / `ワークスペース` などのタイトル文字列を表示せず、アイコン操作とツリー表示を優先しなければならない（MUST）。

#### Scenario: ワークスペースヘッダーからタイトル文字列を除去する

- **WHEN** ユーザーがワークスペースペインを見る
- **THEN** ヘッダーには `Workspace` / `ワークスペース` の文言が表示されない
- **THEN** ワークスペースツリーの表示領域が優先される

### Requirement: ワークスペースヘッダーの操作ボタン整列

アクティブなワークスペースペインの操作ヘッダーは、更新ボタンを先頭側に、全展開・全閉ボタンを末尾側に配置し、フィルター機能を同じヘッダー内で継続利用できなければならない（MUST）。

#### Scenario: ヘッダーの操作グループを描画する

- **WHEN** ユーザーがワークスペースペインを表示する
- **THEN** 更新ボタンはヘッダー行の先頭側に表示される
- **THEN** 全展開ボタンと全閉ボタンはヘッダー行の末尾側に表示される
- **THEN** フィルター機能はヘッダー内で引き続き利用できる
