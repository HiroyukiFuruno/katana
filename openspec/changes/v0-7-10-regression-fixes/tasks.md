## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions. (Override: working directly in `master` per user request)

## 1. 透過度（Theme Transparency）補正機能の復元

- [ ] 1.1 `crates/katana-platform/src/theme/types.rs` に `Rgba::with_offset()` と `ThemeColors::with_contrast_offset()` メソッドを追加し、UIコントラスト設定をアルファ値に適用するロジックを再実装する
- [ ] 1.2 `crates/katana-platform/src/settings/impls.rs` の `effective_theme_colors()` メソッドにて、取得したカラーパレットに対して `ui_contrast_offset` 値を用いて上記メソッドを呼び出し、補正を適用した状態の構造体としてUIへ供給する
- [ ] 1.3 `crates/katana-platform/src/theme/types.rs` のカバレッジ担保のため、`test_rgba_with_offset` ユニットテストを復旧する

### Definition of Done (DoD)

- [ ] (Other task-specific verifiable conditions...)
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Final Verification & Release Work

- [ ] 3.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 3.2 Ensure `make check` passes with exit code 0
- [ ] 3.3 Merge the intermediate base branch (derived originally from master) into the `master` branch (Override: Skipped, work is directly in master)
- [ ] 3.4 Create a PR targeting `master` (Override: Skipped)
- [ ] 3.5 Merge into master (※ `--admin` is permitted) (Override: Skipped)
- [ ] 3.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `v0.7.10`
- [ ] 3.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
