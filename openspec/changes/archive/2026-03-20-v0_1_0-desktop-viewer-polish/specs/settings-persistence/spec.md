## ADDED Requirements

### Requirement: 設定のJSON永続化
全アプリケーション設定をJSONファイルとして永続化する。

#### Scenario: 設定の保存
- **WHEN** ユーザーがテーマ、フォントサイズ等の設定を変更する
- **THEN** 変更がJSONファイル（`~/Library/Application Support/katana/config.json`）に保存される

#### Scenario: 設定の復元
- **WHEN** アプリを起動する
- **THEN** 前回の設定が自動的に復元される

#### Scenario: 設定ファイルが存在しない場合
- **WHEN** 設定ファイルが存在しない状態でアプリを起動する
- **THEN** デフォルト設定が適用され、設定変更時に新規ファイルが作成される

#### Scenario: 設定ファイルが破損している場合
- **WHEN** 設定ファイルのJSONが不正な形式である
- **THEN** デフォルト設定が適用され、ステータスバーに警告メッセージが表示される

### Requirement: SettingsRepository パターン（DIP）
永続化のバックエンドを切り替え可能にするため、Repository パターンと依存性逆転原則（DIP）を適用する。

#### Scenario: ローカルJSON永続化
- **WHEN** デフォルト構成でアプリを起動する
- **THEN** `JsonFileRepository` が使用され、ローカルファイルに設定が保存される

#### Scenario: バックエンドの差し替え
- **WHEN** `SettingsRepository` trait の別実装（例: CloudRepository）を提供する
- **THEN** それを注入するだけで永続化先がCloud Storageに切り替わる（コア/UIの変更不要）
