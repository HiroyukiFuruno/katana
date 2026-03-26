# Implementation Tasks: テーマ設定の構造最適化およびバグフィックス (v0.7.7)

## 1. テーマの内部スキーマ再設計とマイグレーション

- [ ] 1.1 `Theme` 構造体を `PreviewColors`, `CodeColors`, `SystemColors` 等の階層型構造に再定義する。
- [ ] 1.2 `katana-platform/src/settings.rs` に旧バージョンのフラットなカスタムテーマ構造体を定義し、デシリアライズ時のマイグレーション（互換性維持）ロジックを実装する。
- [ ] 1.3 マイグレーション処理の単体テストを追加する。

## 2. 不具合修正と固定値のカスタマイズ化

- [ ] 2.1 実装内で固定値（ハードコード）になっている色情報を洗い出し、`Theme` の適切なグループ（例：`CodeColors`）に抽出・定義する。
- [ ] 2.2 eguiのデフォルトに依存している色情報を洗い出し、`Theme` に追加する。
- [ ] 2.3 カスタムテーマ変更時に「コードの背景色」がプレビューおよびエディタに正しく反映されないバグを修正する。

## 3. 設定画面のUIグルーピング化

- [ ] 3.1 `settings_window.rs` のテーマ設定タブにおいて、フラットに並んでいた色設定項目群を `Preview`, `Code`, `System` それぞれのカテゴリ（アコーディオンまたはセクション）ごとに再配置する。
- [ ] 3.2 ユーザーへのUIスナップショット（画像等）の提示および動作報告。
- [ ] 3.3 ユーザーからのフィードバックに基づくUIの微調整および改善実装。

## 4. プレビューの分割表示レイアウト改善

- [ ] 4.1 テーマ適用状況を確認しやすくするため、プレビューの分割表示用レイアウトの微調整（UIのマージンや振る舞い修正など）を行う。

## 5. Final Verification & Release Work

- [ ] 5.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`.
- [ ] 5.2 Ensure `make check` passes with exit code 0.
- [ ] 5.3 Execute `/openspec-delivery` workflow to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).
- [ ] 5.4 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.7.7`.
- [ ] 5.5 Archive this change by leveraging OpenSpec skills.
