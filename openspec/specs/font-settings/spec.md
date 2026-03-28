## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: フォントサイズ設定
プレビューとエディタのフォントサイズを変更できる。 The system SHALL conform.

#### Scenario: フォントサイズの変更
- **WHEN** ユーザーが設定でフォントサイズを12pxから16pxに変更する
- **THEN** エディタとプレビューのフォントサイズが即座に16pxに反映される

#### Scenario: フォントサイズの範囲
- **WHEN** ユーザーがフォントサイズを設定する
- **THEN** 8px～32pxの範囲で設定可能であり、範囲外の値は設定できない

#### Scenario: フォントサイズの永続化
- **WHEN** ユーザーがフォントサイズを変更する
- **THEN** 設定が保存され、次回起動時に復元される

### Requirement: フォントファミリー設定
プレビューとエディタのフォントファミリーを変更できる。 The system SHALL conform.

#### Scenario: フォントファミリーの変更
- **WHEN** ユーザーが設定でフォントファミリーを変更する
- **THEN** エディタとプレビューのフォントが即座に切り替わる

#### Scenario: デフォルトフォント
- **WHEN** フォントファミリーが未設定
- **THEN** システムデフォルトフォントが使用される

#### Scenario: フォントファミリーの永続化
- **WHEN** ユーザーがフォントファミリーを変更する
- **THEN** 設定が保存され、次回起動時に復元される
