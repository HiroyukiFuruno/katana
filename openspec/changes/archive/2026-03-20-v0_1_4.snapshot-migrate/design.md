## Context

Katana プロジェクトの UI テストは `egui_kittest` を使い、二つの検証手法を併用している:

1. **スナップショットテスト**: `harness.snapshot_options("name", &opts)` で PNG を `tests/snapshots/` に保存し、差分をピクセル比較
2. **セマンティックアサーション**: AccessKit ツリーを利用した `get_by_label()`, `assert_centered()`, `assert_below()` 等による論理検証

`diagram_rendering.rs` と `sample_fixture_tests.rs` の大半はすでにセマンティック方式に移行済み。残る `integration.rs` の 18テストがスナップショットを呼んでいるが、`SNAPSHOT_PIXEL_TOLERANCE = 10000` の設定で実質 fuzzy-pass 状態であり、回帰検知として機能していない。

## Goals / Non-Goals

**Goals:**

- `integration.rs` の全 `snapshot_options()` 呼び出しを除去し、各テストに適切なセマンティックアサーションを追加
- `sample_fixture_tests.rs` に残るスナップショット関連コード（`snapshot_fixture()`, snapshot 参照コメント等）を整理
- `tests/snapshots/` ディレクトリを完全削除
- `Makefile` の `test-update-snapshots` ターゲットを削除
- 全テストが `make check` をパスすることを検証

**Non-Goals:**

- テスト対象の UI ロジック自体の変更（純粋なテスト手法のリファクタリングに限定）
- 新しいテストケースの追加（既存テストの移行のみ）
- `egui_kittest` 依存の除去（Harness / AccessKit は引き続き使用）

## Decisions

- **セマンティックアサーションへの統一**
  - *Rationale*: `diagram_rendering.rs` で確立済みのパターン（`get_by_label`, `query_all_by_value`, `assert_*` ヘルパー）が十分な回帰検知能力を持つことが実証済み。ピクセル比較は環境差異に脆弱で、10000px の tolerance は事実上のノーチェックと同等。
- **段階的移行ではなく一括移行**
  - *Rationale*: スナップショットは 1テストのみ残しても `snapshots/` ディレクトリとPNGの管理コストが発生する。既に確立されたパターンがあるため、一括移行のリスクは低い。
- **各テストの移行方針**
  - snapshot のみで終わるテスト → 適切な AccessKit クエリ（`get_by_label`, `get_by_value`）と状態アサーション（`assert_eq!`）に置換
  - snapshot + セマンティック検証を併用するテスト → snapshot 呼び出しのみ削除、既存セマンティック検証を維持

## Risks / Trade-offs

- **[Risk] スナップショットでしか検知できないレイアウト回帰**
  - AccessKit はテキストラベルの存在と大まかな位置は検証できるが、ピクセル精度のレイアウト崩れ（余白、フォントサイズ等）は検知不可。
  - **Mitigation**: 現在の `SNAPSHOT_PIXEL_TOLERANCE = 10000` では同等の精度のレイアウト検知は既に不可能。`sample_fixture_tests.rs` の `assert_centered`, `assert_below`, `assert_gap_at_least` で実用十分なレイアウト回帰防止を実現済み。
- **[Risk] 未知のUIバグの見逃し**
  - **Mitigation**: 各テストにセマンティックアサーションを追加するため、「何も検証しないテスト」は発生しない。ラベル存在チェック + 状態アサーションで主要な回帰は検知可能。
