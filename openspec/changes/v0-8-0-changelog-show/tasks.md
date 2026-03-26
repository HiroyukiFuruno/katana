# Implementation Tasks: アプリ内リリースノート表示機能 (v0.8.0)

## 1. ヘルプメニューUIの拡張

- [ ] 1.1 `shell_ui.rs` における `Help` メニューの配下に `Release Notes` を追加する。
- [ ] 1.2 サブメニューとして `Latest` と `All` を配置し、各アクションの発火をシェル等のイベントシステムへ通知する仕組みを構築する。

## 2. CHANGELOGのフェッチ・パース機能の実装

- [ ] 2.1 HTTPクライアントを用い、GitHub上の `CHANGELOG.md` または `CHANGELOG_ja.md` （ロケールに応じて切り替え）をフェッチする非同期プロセスを実装する。
- [ ] 2.2 オフライン時やフェッチ失敗時のエラーハンドリング（タイムアウト等）を適切に設計する。
- [ ] 2.3 取得したマークダウンテキストを、バージョン（`## [vX.Y.Z]` や `# [vX.Y.Z]` 等の形式）をデリミタとして分割・構造化するパーサ（`ReleaseNoteEntry { version, body }`）を実装する。

## 3. リリースノート共通UIコンポーネントの構築

- [ ] 3.1 バージョンごとの `ReleaseNoteEntry` のリストを受け取り、アコーディオン形式で表示するウィジェットを実装する。
- [ ] 3.2 `Latest` のみ開かれた状態（展開）にし、他のバージョンを折りたたんだデフォルト状態にする。
- [ ] 3.3 ウィジェットの引数に、表示スコープ（全件、最新N件、起点〜終点バージョンの差分）をフィルタリングするロジックまたはオプション機能を統合する。

## 4. 更新確認（Update Check）画面への差分表示の統合

- [ ] 4.1 アプリのアップデート通知（更新チェック結果画面）に、新バージョンのリリースノートを案内するためのボタンまたは表示領域を追加する。
- [ ] 4.2 構築した共通UIコンポーネントに「現在バージョン 〜 新バージョン」のフィルタオプションを渡し、差分リリースノートのみをリストアップ描画させる。

## 5. UIスナップショットとフィードバック対応

- [ ] 5.1 ユーザーへのUIスナップショット（画像等）の提示および動作報告。
- [ ] 5.2 ユーザーからのフィードバックに基づくUIの微調整（余白、フォントサイズ、遅延ローディング等）の改善実装。

## 6. Final Verification & Release Work

- [ ] 6.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`.
- [ ] 6.2 Ensure `make check` passes with exit code 0.
- [ ] 6.3 Execute `/openspec-delivery` workflow to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).
- [ ] 6.4 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.8.0`.
- [ ] 6.5 Archive this change by leveraging OpenSpec skills.
