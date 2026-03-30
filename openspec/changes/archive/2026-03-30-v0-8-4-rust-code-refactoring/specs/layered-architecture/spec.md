## ADDED Requirements

### Requirement: 4レイヤーごとの内部階層設計

各レイヤー（core, linter, platform, ui）内部をサブモジュールに分割し、1ファイル1責務の原則を適用する (MUST).

- 推奨上限: 150行（テスト除外）
- ハードリミット: 200行（テスト除外）
- 型定義と実装の物理的分離パターン（`types.rs` / `impls.rs`）を適用する
- `pub use` による re-export で外部APIの互換性を維持する

#### Scenario: Verify modularity
- **Given** a layer structure
- **When** code is organized
- **Then** it follows the 1 file 1 responsibility rule and types/impls pattern.

### Requirement: SOLID原則違反の解消

以下のSOLID違反を解消しなければならない (MUST).

- **SRP違反**: `KatanaApp`（God Object: 40+メソッド）、`AppState`（57フィールド）の解体
- **SRP違反**: `shell_ui.rs`（5,118行、46関数）、`shell.rs`（3,144行）の責務分離
- **OCP違反**: メニュー構成・タブ追加が既存コード直接修正を必要とする構造の改善
- **ISP違反**: `PreviewPane` の全責務保持の解消
- **DIP未整備**: レイヤー間の具象型直接参照 → trait抽象の検討

#### Scenario: Verify SOLID
- **Given** existing God Objects and tight coupling
- **When** refactored
- **Then** SRP is respected and massive files are split.

### Requirement: UIは再利用可能な自己完結コンポーネントへ分解する

UIレイヤーの分割は、単なる free function のファイル移設であってはならず、Reactコンポーネントのように再利用可能で再現性の高い自己完結コンポーネントとして構成しなければならない (MUST)。

- `Props` に相当する入力境界を持つ
- 描画は `show(...)` などの明示的なコンポーネント入口に集約する
- ユーザー操作は typed response / action として親へ返す
- 親子 compose は最小限の依存だけを受け渡す

#### Scenario: `shell_ui.rs` から UI を抽出する

- **WHEN** `shell_ui.rs` のメニュー、ワークスペース、タブ、モーダル等を分割する
- **THEN** 分割先は component `struct` または同等の typed UI module として定義される
- **THEN** 既存の `render_*` free function を別ファイルへ再配置しただけの状態は完了とみなされない

#### Scenario: `settings_window.rs` / `preview_pane_ui.rs` / `widgets.rs` を分割する

- **WHEN** 設定タブ、プレビューセクション、共通ウィジェットを分割する
- **THEN** 各モジュールは最小限の props と typed response を持つ再利用可能コンポーネントになる
- **THEN** 親コンポーネントは巨大な state 全体をそのまま子へ渡さない

#### Scenario: UIリファクタリング完了を判定する

- **WHEN** katana-ui レイヤーのリファクタリングを完了判定する
- **THEN** `menu`, `workspace`, `tab_bar`, `settings`, `preview`, `modals`, `widgets` の主要導線が自己完結コンポーネントとして構成されている
- **THEN** representative な UI interaction tests で component 境界越しの操作が検証されている

### Requirement: レイヤー間依存方向の維持

リファクタリング後も以下の依存方向を維持しなければならない (MUST)。

```text
katana-core    → (外部クレートのみ)
katana-linter  → (独立)
katana-platform → katana-core
katana-ui       → katana-core, katana-platform
```

逆方向の依存（例: coreがuiに依存する）は禁止する。

#### Scenario: Verify Dependency Direction
- **Given** the 4 crates
- **When** analyzed using `cargo tree`
- **Then** core does not depend on ui, and dependencies only flow inward.
