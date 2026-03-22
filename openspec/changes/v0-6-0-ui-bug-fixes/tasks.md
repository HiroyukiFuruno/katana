## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. P0 CPU無限ループ および タブ切り替えバグ修正

- [x] 1.1 `KatanaSvgLoader::load` が失敗した場合にエラーを返し、キャッシュさせるようにする (`LoadError::NotSupported` の利用)
- [x] 1.2 `shell.rs` において、`tab.hash` （キャッシュ）を判定し、同一ハッシュなら早期リターンするよう最適化 (CPU100%バグ修正)

### Definition of Done (DoD)

- [x] CPUが100%に張り付く現象が消失し、`tests::svg_loader` などが通過する
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. P1/P3/P4 UI レンダリングバグ修正

- [x] 2.1 `needs_splash` フラグを使い、UI初期化ではなく「更新（初回フレーム）」タイミングでSplashタイマーを開始させる
- [x] 2.2 Blockquote 上下の不要な表示改行を削除する
- [x] 2.3 `KatanaApp` から不要にエラーが出る `bytes://icon/copy.svg` などを事前登録し、アイコンエラーを解消する

### Definition of Done (DoD)

- [x] 起動時に正しくスプラッシュが表示され、テスト `make test` にも退行が生じていない
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Final Verification & Release Work

- [x] 3.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [x] 3.2 Ensure `make check` passes with exit code 0
- [ ] 3.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 3.4 Create a PR targeting `master`
- [ ] 3.5 Merge into master (※ `--admin` is permitted)
- [ ] 3.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.6.0`
- [ ] 3.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
