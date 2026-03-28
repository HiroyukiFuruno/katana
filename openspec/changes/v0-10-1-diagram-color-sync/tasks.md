## Definition of Ready (DoR)

- [x] v0.10.1の仕様（Mermaid/PlantUMLのグローバルカラーキャッシュの動的化）が確定し合意されていること。

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. Requirement Refinement

- [ ] 1.1 DiagramColorPresetのライフサイクルとOnceLockの廃止要件を整理
- [ ] 1.2 テーマの色情報（ui.visuals().text_color()等）をプレビューペインやDiagramレンダラまでどのように伝達するかの構造設計
- [ ] ### Definition of Done (DoD)
- [ ] アーキテクチャ図または詳細設計が `propose.md` に記載されレビューを通過すること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. Diagram Color Dynamic Resolution Implementation

- [ ] 2.1 PreviewPaneの`DiagramColorPreset`から`OnceLock`を排除し、レンダリング毎に解決を試みるか、依存関係パイプラインを通すように修正
- [ ] 2.2 `shell.rs`等のThemeStoreへの依存経路を通じて、現在の`preview.text`または`preview.border`の実際の色を取得するように連携
- [ ] 2.3 MermaidコンポーネントにおけるCLI側へのHex値パッシング仕様の修正
- [ ] 2.4 PlantUMLコンポーネントにおけるCLI側へのHex値パッシング設定の修正
- [ ] ### Definition of Done (DoD)
- [ ] Mermaid等で表示されるテキストの色がテーマ変更で動的に完全に追従できるようになること
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Final Verification & Release Work

- [ ] 3.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 3.2 Ensure `make check` passes with exit code 0
- [ ] 3.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 3.4 Create a PR targeting `master`
- [ ] 3.5 Merge into master (※ `--admin` is permitted)
- [ ] 3.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.10.1`
- [ ] 3.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
