## 1. Core Platform (Cache I/O)

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

- [ ] 1.1 現行 `DefaultCacheService` の read/write workload を計測し、問題が「JSON の遅さ」ではなく「全量再シリアライズ・全量書き込み・線形探索」にあることを benchmark / tracing で確認する
- [ ] 1.1.1 benchmark / tracing の結果を `design.md` に要約し、前提が崩れた場合は 1.3 へ進む前に proposal / design / spec / tasks を更新する
- [ ] 1.2 `workspace_tabs` / `diagram` などの persistent key namespace を整理し、内部ロジック用の key codec / filename codec を定義する
- [ ] 1.2.1 filename hash を address 用 codec と位置づけ、各 file に `storage_version` と canonical key metadata を含める entry envelope を定義する
- [ ] 1.2.2 `workspace_tabs` と `diagram` の canonical key field を文書化し、追加 namespace が必要と判明した場合は spec を先に更新する
- [ ] 1.2.3 `CacheFacade` の `&str` key を canonical raw key として扱う helper を定義し、call site が直接 opaque key を組み立てない方針を実装に落とす
- [ ] 1.3 `DefaultCacheService` を改修し、runtime の正規 persistent backend を `cache.json` から per-key file store へ切り替える
- [ ] 1.4 `get_persistent` において、namespace-aware key から安全なファイル名を導出し、必要に応じて `kv` 配下から key 単位で読み込むロジックを実装する
- [ ] 1.5 `set_persistent` において、指定キーのデータのみを `kv` 配下へ同期し、他キーの再シリアライズを行わない処理を実装する
- [ ] 1.6 初期化（`new`）時、旧 `cache.json` が存在する場合はその中身を KVS へ one-shot migration し、完了後は runtime backend として併用しない構成にする
- [ ] 1.6.1 migration の保証対象を `workspace_tabs` などのユーザー状態 namespace に明示し、旧 `cache.json` はその移送成功前に破棄しない
- [ ] 1.6.2 `diagram` cache は再生成可能データとして扱い、旧 `diagram_<hash>` は migration 対象外としてスキップする方針を実装する
- [ ] 1.6.3 migration を partial state から再実行しても壊れないよう、temp file / atomic rename / retry 前提の処理を実装する
- [ ] 1.7 `PersistentData.entries` / `Vec<(String, String)>` 依存を見直し、オンメモリ探索構造も key 設計に合う形へ整理する
- [ ] 1.8 `workspace.rs` の `workspace_tabs:*` call site を canonical raw key helper に置き換える
- [ ] 1.9 `renderer.rs` / `core_render.rs` の `diagram_<hash>` write/read path を廃止し、canonical raw key + storage filename codec 構成へ置き換える

### Definition of Done (DoD)
- [ ] benchmark / tracing の結果が `design.md` に反映され、前提との一致または差分が説明されている
- [ ] canonical key / filename codec / entry envelope / migration 方針が proposal・design・spec で矛盾なく読める
- [ ] `workspace_tabs` は upgrade で保全対象、`diagram` は best-effort であることが実装と文書で一致している
- [ ] `diagram_<hash>` が legacy key であり、新規 write path では使われていないことが実装と文書で一致している
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. Testing & Verification

### Definition of Ready (DoR)

- [ ] Ensure the previous task completed its full delivery cycle: self-review, recovery (if needed), PR creation, merge, and branch deletion.
- [ ] Base branch is synced, and a new branch is explicitly created for this task.

- [ ] 2.1 `cache/default.rs` と関連テストを per-key file store / key codec 構成に合わせて修正し、migration・lookup・write path を網羅する
- [ ] 2.2 `workspace_tabs` の保存復元、diagram cache の保存読込、invalid key / missing file path をそれぞれ namespace 単位で検証する
- [ ] 2.2.1 restart 後に同じ logical key から同じ filename を再計算して復元できること、および file content の metadata から key を検証できることを確認する
- [ ] 2.2.2 `workspace_tabs` と `diagram` の caller が helper から canonical raw key を生成し、storage 層で filename hash が決まることを確認する
- [ ] 2.3 既存のUI操作（タブ切替、起動時の状態復元、画像キャッシュ復元等）による連携テストを実行し、デグレがないかを検証する
- [ ] 2.4 キャッシュ全体クリア（`clear_all_directories_in`）が新しい `kv` ディレクトリ構造でも正常に作動し、孤立したファイルが一切残らないことを確認する
- [ ] 2.5 benchmark / tracing の結果を記録し、改善の中心が JSON 置換ではなく storage unit と key 設計であることを確認する
- [ ] 2.6 upgrade compatibility と downgrade 非保証の境界が design / spec / release-facing note のいずれかで明文化されていることを確認する
- [ ] 2.7 migration の partial failure / retry / restart をテストし、`workspace_tabs` を失わずに再収束できることを確認する
- [ ] 2.7.1 旧 `diagram_<hash>` entry が存在しても migration が失敗せず、diagram cache が再生成で回復することを確認する
- [ ] 2.8 実装途中に design 前提と乖離した点があれば、関連 artifact が先に更新されていることを確認する

### Definition of Done (DoD)
- [ ] test と manual verification で restart / migration / clear / restore が一通り確認されている
- [ ] release-facing note または同等の記録に、upgrade 保証と downgrade 非保証が残されている
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Final Verification & Release Work

- [ ] 3.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 3.2 Ensure `make check` passes with exit code 0
- [ ] 3.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 3.4 Create a PR targeting `master`
- [ ] 3.5 Merge into master (※ `--admin` is permitted)
- [ ] 3.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.10.0`
- [ ] 3.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
