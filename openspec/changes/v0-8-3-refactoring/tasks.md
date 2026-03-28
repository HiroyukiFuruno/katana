# タスク一覧: Settings アーキテクチャ分離

- [ ] **Phase 1: 依存関係の洗い出しと準備**
  - 現在の `settings.rs` および各 `migration_*.rs` の状態を確認する。
  - `theme.rs` のリファクタリング時と同様に、既存のコンパイルを維持しながら段階的に移行する方針を確立する。

- [ ] **Phase 2: ディレクトリとモジュールの枠組み作成**
  - `crates/katana-platform/src/settings/` ディレクトリを作成する。
  - `mod.rs`, `types.rs`, `impls.rs`, `service.rs`, `repository.rs` を空ファイルとして作成。
  - `migration/` ディレクトリを作成し、既存のマイグレーションファイルを移動させる。

- [ ] **Phase 3: 型の移行 (types.rs)**
  - `AppSettings`, `WindowSettings`, `EditorSettings`, `TerminalSettings`, `ThemeSettings` など全ての `struct` および `enum` を `types.rs` へ移動。
  - `#[derive(...)]` 等の必要なインポートを整備する。

- [ ] **Phase 4: 実装の移行 (impls.rs)**
  - 各種設定型の `impl Default for ...` やカスタムメソッドの実装を `impls.rs` へ移動。
  - `types.rs` から正しく型を参照できるように `use` パスを整理する。

- [ ] **Phase 5: サービスおよびリポジトリ層の独立**
  - `SettingsService` および関連メソッド群を `service.rs` へ移動。
  - ファイル I/O (`load`, `save` など) に関わるロジックを `repository.rs` へ移動。

- [ ] **Phase 6: `katana-ui` 等からの参照パス解決**
  - 新モジュールツリーに合わせて、`crates/katana-ui` 側の利用箇所（`use katana_platform::settings::...`）における名前解決エラーをすべて解消する。
  - `cargo check` にて全体が正常にビルドできることを検証する。

## Checklists

### 成果物チェックリスト

- [ ] `settings.rs` の肥大化が解消され、分離されたディレクトリ構造(`settings/`)へ移行された
- [ ] 型 (`types.rs`) と実装 (`impls.rs` 等) が分離され、コーディング規約に沿った状態となった
- [ ] 既存のすべてのテストおよび UI がこれまで通り動作する (デグレードなし)

### ブランチ・リリースルール準拠チェック

- [ ] 作業はアサインされた専用のフィーチャーブランチで行われている
- [ ] 日本語でのコミットメッセージ等、プロジェクト規約を満たしている
