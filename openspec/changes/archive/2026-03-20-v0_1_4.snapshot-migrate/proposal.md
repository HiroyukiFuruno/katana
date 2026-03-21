## Why

`integration.rs` の 18テストが `harness.snapshot_options()` で PNG スナップショットを撮影しているが、以下の問題がある:

1. **CI が無意味に遅い**: スナップショット撮影・比較は egui レンダリングを伴い、`make check` の実行時間を増大させている
2. **環境差異で脆弱**: フォントヒンティング・アンチエイリアシングが Local / CI 間で異なるため、`SNAPSHOT_PIXEL_TOLERANCE = 10000` という非常に緩いしきい値を設定しており、実質的に回帰検知能力がない
3. **56枚の PNG が git を汚す**: `tests/snapshots/` に 56ファイル (~10MB) の画像がリポジトリに存在し、diff レビューも不可能
4. **すでに代替手段が確立済み**: `diagram_rendering.rs` と `sample_fixture_tests.rs` では AccessKit ベースのセマンティックアサーション（`get_by_label`, `assert_centered`, `assert_below` 等）への移行が完了しており、同等以上の回帰検知を実現している

## What Changes

- `integration.rs` の全 `snapshot_options()` 呼び出しを削除し、各テストを **セマンティックアサーション（AccessKit クエリ + 状態アサーション）** に置換
- `sample_fixture_tests.rs` に残存する `snapshot_fixture()` 関数および未使用の snapshot 関連コードを整理
- `tests/snapshots/` ディレクトリの全 PNG ファイルを削除
- `Makefile` から `test-update-snapshots` ターゲットを削除
- `Cargo.toml` から不要になった snapshot 関連依存の整理（該当があれば）

## Capabilities

### New Capabilities

- なし（既存テストの検証品質を維持しつつ高速化する内部リファクタリング）

### Modified Capabilities

- `integration.rs`: スナップショット依存テスト → セマンティックアサーションテストに移行
- `sample_fixture_tests.rs`: snapshot_fixture → セマンティック検証のみ（既に一部移行済み）

## Impact

- **CI 高速化**: スナップショット撮影・比較が不要になり `make check` / `make test-integration` が高速化
- **リポジトリ軽量化**: 56枚の PNG ファイル (~10MB) を削除
- **テスト安定性向上**: 環境依存のピクセル比較を排除し、決定論的なセマンティック検証のみに
- **開発体験改善**: `UPDATE_SNAPSHOTS=true` の手動更新ワークフローが不要に
