## ADDED Requirements

### Requirement: vendor/egui_commonmark の依存管理のサブツリー化

公式アップストリームとの連携性を高めるため、直書きのソースコード管理から `git subtree` ベースへ移行すること。

#### Scenario: 独自パッチの隔離と適用

- **WHEN** アップストリームから `egui_commonmark` を同期する
- **THEN** Katana にて独自適用していたパッチ内容が明確に分離・再適用され、依存関係がクリーンに維持される
