## ADDED Requirements

### Requirement: 4レイヤーごとの内部階層設計

各レイヤー（core, linter, platform, ui）内部をサブモジュールに分割し、1ファイル1責務の原則を適用する。

- 推奨上限: 150行（テスト除外）
- ハードリミット: 200行（テスト除外）
- 型定義と実装の物理的分離パターン（`types.rs` / `impls.rs`）を適用する
- `pub use` による re-export で外部APIの互換性を維持する

### Requirement: SOLID原則違反の解消

以下のSOLID違反を解消する：

- **SRP違反**: `KatanaApp`（God Object: 40+メソッド）、`AppState`（57フィールド）の解体
- **SRP違反**: `shell_ui.rs`（5,118行、46関数）、`shell.rs`（3,144行）の責務分離
- **OCP違反**: メニュー構成・タブ追加が既存コード直接修正を必要とする構造の改善
- **ISP違反**: `PreviewPane` の全責務保持の解消
- **DIP未整備**: レイヤー間の具象型直接参照 → trait抽象の検討

### Requirement: レイヤー間依存方向の維持

リファクタリング後も以下の依存方向を維持する：

```
katana-core    → (外部クレートのみ)
katana-linter  → (独立)
katana-platform → katana-core
katana-ui       → katana-core, katana-platform
```

逆方向の依存（例: coreがuiに依存する）は禁止する。
