## ADDED Requirements

### Requirement: テーマ設定（Dark / Light）
Dark / Light の2種類のテーマを提供し、ユーザーが切り替え可能にする。デフォルトはDark。

#### Scenario: テーマの切り替え
- **WHEN** ユーザーが設定メニューからテーマをDarkからLightに変更する
- **THEN** アプリ全体の配色がLightテーマに切り替わる

#### Scenario: デフォルトテーマ
- **WHEN** アプリを初回起動する
- **THEN** Darkテーマが適用されている

#### Scenario: テーマ設定の永続化
- **WHEN** ユーザーがテーマを切り替える
- **THEN** 選択したテーマが設定に保存され、次回起動時に復元される

#### Scenario: テーマ切替の即時反映
- **WHEN** テーマを切り替える
- **THEN** 再起動なしで即座にUI全体に反映される
