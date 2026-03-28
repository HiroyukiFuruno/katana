# Implementation Tasks: アプリ内リリースノート表示機能 (v0.8.0)

## 1. アプリ起動時のバージョン管理・自動表示フック

- [ ] 1.1 アプリの起動時に `previous_version` （設定等に保存されている前回起動時のバージョン）と `current_version` を比較する仕組みを導入する。
- [ ] 1.2 `current_version > previous_version` と判定された場合（アップデート後の初回起動）、バックグラウンドでCHANGELOGの取得・自動表示フローをトリガーし、表示後に `previous_version` を更新する。

## 2. CHANGELOGのフェッチ・パース機能の実装

- [ ] 2.1 HTTPクライアントを用い、GitHub上の `CHANGELOG.ja.md` （日本語環境）または `CHANGELOG.md` （それ以外全て）をフェッチする非同期プロセスを実装する。
- [ ] 2.2 ネットワークエラーやオフライン時に対するフォールバック（既存のキャッシュ表示、またはエラートースト通知等）を設計する。
- [ ] 2.3 取得したマークダウンテキストを、バージョンヘッダー（例 `## [0.8.0]` 等）をデリミタとして分割・パースし、各セクションごとに抽出する処理を実装する。

## 3. アコーディオン付きMarkdownの動的生成とキャッシュ化

- [ ] 3.1 抽出されたバージョンブロック配列に対し、以下のロジックで `<details>` と `<summary>` タグを追加し、一本のMarkdown文字列として構築する。
  - `previous_version` より新しく、`current_version` 以下のバージョンブロックについては `<details open>` にする。
  - 手動アクセス時等の平時（`current_version == previous_version`時）や、過去のバージョン群は全て `<details>` にする。
- [ ] 3.2 生成したMarkdownファイルを、アプリのローカルディレクトリに専用キャッシュ（例: `release_notes_cache.md`）として保存し、不要な毎回の再生成を省く機構を実装する。

## 4. エディターの新規タブにおける表示とUI連携

- [ ] 4.1 キャッシュとして生成されたMarkdownファイルを読み取り、KatanA上で新規のエディタータブとして開く。
- [ ] 4.2 その際の当該タブ名を、ObsidianのUIを参考にし `📄 リリースノート 1.12.7` / `📄 Release Notes 1.12.7` （アイコン＋言語に応じたタイトル＋バージョン）となるよう制御を組み込む。
- [ ] 4.3 `shell_ui.rs` 等における `Help` メニューの配下に `Release Notes` を追加し、ユーザーが任意のタイミングでキャッシュを開けるようにする（このときは全件closeで生成・表示）。
- [ ] 4.4 アプリのアップデート通知（更新チェック）ダイアログに、リリースノートタブを開くための導線（ボタンやリンク等）を追加する。

## 5. UIスナップショットとフィードバック対応

- [ ] 5.1 ユーザーへのUIスナップショット（画像等）の提示および動作報告。
- [ ] 5.2 ユーザーからのフィードバックに基づく微調整（アコーディオン内余白、タブ名アイコンの見栄え、TOCジャンプ追加等）の改善実装。

## 6. Final Verification & Release Work

- [ ] 6.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md`.
- [ ] 6.2 Ensure `make check` passes with exit code 0.
- [ ] 6.3 Execute `/openspec-delivery` workflow to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).
- [ ] 6.4 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.8.0`.
- [ ] 6.5 Archive this change by leveraging OpenSpec skills.
