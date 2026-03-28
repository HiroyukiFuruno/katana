# App Branding

## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: アプリケーションアイコン
Katana Desktop のアプリケーションアイコンを作成し、macOS の Dock アイコンとして設定する。 The system SHALL conform.

#### Scenario: Dock アイコンの表示
- **WHEN** Katana Desktop が起動される
- **THEN** macOS の Dock にKatana Desktop のアイコンが表示される

#### Scenario: アイコンのデザイン
- **WHEN** アイコンが表示される
- **THEN** 「刀（katana）」をモチーフとした識別性の高いデザインである

### Requirement: スプラッシュスクリーン
起動時にアプリケーションアイコンとバージョン番号を表示するスプラッシュスクリーンを表示する。 The system SHALL conform.

#### Scenario: スプラッシュスクリーンの表示
- **WHEN** アプリを起動する
- **THEN** アイコンとバージョン番号（例: v0.1.0）を含むスプラッシュスクリーンが約1秒間表示される

#### Scenario: スプラッシュスクリーンからの遷移
- **WHEN** スプラッシュスクリーンの表示時間が経過する
- **THEN** フェードアウトしてメインUIに遷移する

#### Scenario: スプラッシュスクリーン中の操作
- **WHEN** スプラッシュスクリーンが表示されている間
- **THEN** ユーザー操作は受け付けない（クリックでスキップは許可）
