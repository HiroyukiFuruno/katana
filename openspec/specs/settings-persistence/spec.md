## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: 設定のJSON永続化
全アプリケーション設定をJSONファイルとして永続化する。 The system SHALL conform.

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
永続化のバックエンドを切り替え可能にするため、Repository パターンと依存性逆転原則（DIP）を適用する。 The system SHALL conform.

#### Scenario: ローカルJSON永続化
- **WHEN** デフォルト構成でアプリを起動する
- **THEN** `JsonFileRepository` が使用され、ローカルファイルに設定が保存される

#### Scenario: バックエンドの差し替え
- **WHEN** `SettingsRepository` trait の別実装（例: CloudRepository）を提供する
- **THEN** それを注入するだけで永続化先がCloud Storageに切り替わる（コア/UIの変更不要）

### Requirement: Type-safe Extensible Settings Lists
The domain model storing supplementary user configurations (`extra`) SHALL be represented strictly as a List of structurally typed configurations (`Vec<ExtraSetting>`).

#### Scenario: Persisting generic extensions
- **WHEN** application needs to store random key-value extras
- **THEN** it serializes to a JSON array of `{"key": "...", "value": "..."}` objects instead of an implicit Map `{ "key": "value" }`.

### Requirement: Automatic Format Migration
The application SHALL migrate legacy JSON objects into JSON arrays immediately upon load when encountering legacy `v0.1.3` (or older) settings structures.

#### Scenario: User upgrades from v0.1.3 to v0.1.4
- **WHEN** the application boots and discovers a legacy `settings.json` where `extra` is an object
- **THEN** the migration runner (`Migration0_1_3_to_0_1_4`) safely transforms it into the new Array schema before failing structural parsing.
