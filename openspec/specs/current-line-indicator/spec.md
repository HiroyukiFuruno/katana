# Current Line Indicator

## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: カレント行表示
エディタ（CodeOnly / Split モード）で、カーソルが位置する行をハイライト表示する。 The system SHALL conform.

#### Scenario: カレント行のハイライト
- **WHEN** ユーザーがエディタでテキストを編集する
- **THEN** カーソルが位置する行の背景色が他の行と異なり、現在位置が視覚的に識別可能

#### Scenario: カレント行の行番号表示
- **WHEN** エディタが表示されている
- **THEN** 各行の左端に行番号が表示され、カレント行の行番号は強調表示される

#### Scenario: スクロール時のカレント行追従
- **WHEN** カーソル位置がビューポート外にスクロールされる
- **THEN** カーソル位置に戻る操作（ショートカット等）でカレント行にスクロールバックする
