## Why

KatanAの永続化キャッシュ（`cache.json`）は、現在の実装では全 persistent key を 1 つの `Vec<(String, String)>` に保持し、`set_persistent` のたびに全体を再シリアライズして単一ファイルへ丸ごと書き戻しています。問題の本質は Rust の JSON 処理速度そのものではなく、小さなワークスペース状態と大きな図解キャッシュが同じ保存単位に混在していること、そしてキー設計が文字列ベースで曖昧なため内部ロジックの分離・削除・移行を組みにくいことです。

## What Changes

- **runtime の単一ファイル依存を廃止**: `cache.json` は移行元としてのみ扱い、runtime の正規保存先はキー単位の個別ファイルへ統一します。
- **namespace-aware な key 設計の導入**: `workspace_tabs`, `diagram` などの用途別 namespace を明示し、内部ロジックから型的に扱いやすい key codec / filename codec を導入します。ファイル名は安全な hash を使っても、ファイル内容には storage version と canonical key metadata を保持します。
- **call site の canonical key 化**: storage 層だけで opaque key を解釈しようとはせず、`workspace_tabs` と `diagram` の caller は canonical raw key を生成して `CacheFacade` へ渡すように変更します。既存の `diagram_<hash>` は legacy key として扱い、新形式への upgrade 対象にはしません。
- **KVSディレクトリ構造の導入**: `~/.cache/KatanA/kv` 配下に namespace ごとのファイルを配置し、該当キーのデータだけを読み書きする構造へ移行します。
- **I/O 増幅の除去**: 小さな状態変更で巨大なキャッシュ全体を書き戻さないようにし、書き込み単位をキー単位へ分解します。
- **互換性対応 (マイグレーション)**: 既存の `cache.json` は初回起動時に新形式へ移送して退避し、その後の runtime 経路では使用しません。upgrade 時には workspace state の引き継ぎを保証対象として扱い、旧 `diagram_<hash>` は metadata を復元できないため新形式への移送対象とせず、必要に応じて再生成します。
- **互換性方針の明文化**: v0.10.0 では「旧 `cache.json` から新 store への upgrade」は保証対象とし、new store から旧版アプリへの downgrade 互換までは保証しない前提を明記します。
- **検証の明文化**: ベンチマークと実 workload 計測を追加し、「JSON が遅い」のではなく「全量再書き込みとキー設計」が問題であることを検証可能にします。

## Capabilities

### New Capabilities

- `cache-io`: per-key file store と namespace-aware key 設計による永続化キャッシュ基盤

### Modified Capabilities

## Impact

- `crates/katana-platform/src/cache/default.rs`: `DefaultCacheService` の persistent path と保存単位の全面刷新
- `crates/katana-platform/src/cache/mod.rs`: `PersistentData` / key 取り扱いの見直し
- `crates/katana-ui/src/app/workspace.rs`: `workspace_tabs:*` key の codec と restore/save 経路
- `crates/katana-ui/src/preview_pane/core_render.rs`
- `crates/katana-ui/src/preview_pane/renderer.rs`: `diagram_*` key の codec と invalidation 単位
- キャッシュ一括削除処理は `kv` ディレクトリを対象に含め、旧 `cache.json` は migration 後に runtime から外れる
