# Tasks: v0.7.0 In-App Auto Update

## Definition of Ready (DoR)

- [x] Proposal approved
- [ ] Spec defined

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. アプリ内自動更新機能の実装 (Core Update Logic)

- [x] 1.1 `katana-core` へのバージョンチェック機能実装 (GitHub Releases API連携。ローカル検証用APIオーバーライド機能を含む)
- [x] 1.2 `katana-core` への .zip ダウンロード・一時展開・Relauncher（ヘルパースクリプト）生成機能の実装
- [ ] 1.3 `katana-core` から Relauncher をバックグラウンド起動し、自身を終了させるフローの構築（置換・`xattr -cr`・テンポラリのクリーンアップ・再起動処理を委譲）
- [ ] 1.4 `katana-core` へのチェック頻度や状態管理・エラーハンドリング機能の実装
- [ ] 1.5 ローカルHTTPサーバーとダミーファイルを用いた、実リリース前の完全ローカル疑似E2Eテストの実行と検証

### Definition of Done (DoD)

- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. 更新通知・設定 UI の実装 (UI / View)

- [ ] 2.1 `katana-ui` への更新通知ダイアログ（CHANGELOG差分サマリの表示）実装
- [ ] 2.2 `katana-ui` への「今すぐ更新」「後で」「このバージョンをスキップ」操作フロー実装
- [ ] 2.3 `katana-ui` への更新完了後の再起動プロンプト表示実装
- [ ] 2.4 `katana-ui` 内の環境設定画面への「起動時チェック」のON/OFFおよび頻度設定UIの追加
- [ ] 2.5 ユーザーへの実装済みUIスナップショット（画面キャプチャ）の提示および動作報告
- [ ] 2.6 ユーザーからの確認・フィードバックに基づくUIの微調整および改善

### Definition of Done (DoD)

- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 3. CI/CD 拡張 (Workflow)

- [ ] 3.1 `.github/workflows/release.yml` に macOS 用 `.app` バンドルを圧縮した自動更新用 `.zip` のビルド生成・アップロード手順を追加

### Definition of Done (DoD)

- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. Final Verification & Release Work

- [ ] 4.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [ ] 4.2 Ensure `make check` passes with exit code 0
- [ ] 4.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [ ] 4.4 Create a PR targeting `master`
- [ ] 4.5 Merge into master (※ `--admin` is permitted)
- [ ] 4.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.7.0`
- [ ] 4.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`
