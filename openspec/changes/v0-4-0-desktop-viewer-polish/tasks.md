## Definition of Ready (DoR)

- **前提条件**: `desktop-viewer-polish-v0.3.0` の内容が `main` ブランチにマージされていること。
- **タスクの順序性**:
  - `タスク4 (画像・ダイアグラムのビューアコントロール追加)` は `タスク1 (ローカル画像のプレビュー)` が完了し、画像レンダリング基盤が整ってから着手すること。

## Branch Rule

タスクグループ（##単位）= 1セッションで以下のサイクルを回す:

1. **ベースブランチ作成**: 最初に `desktop-viewer-polish-v0.4.0` を `master` から作成する
2. **タスクブランチ作成**: 各タスクごとの作業は `1` のブランチから `desktop-viewer-polish-v0.4.0-task{N}` を派生させる
3. **実装 / PR / セルフレビュー / マージ**: `/opsx-apply` (マージ先は `master` ではなく `1` のブランチとする。マージ完了後は `1` のベースブランチへ切り替え、最新化を行う)

---

## 1. ローカル画像の遅延読み込みプレビュー

- [x] 1.1 comrak AST から画像ノードのローカル相対パスを検出・解決する
- [x] 1.2 バックグラウンドで全画像を読み込み、完了するまでプレースホルダーを表示する
- [x] 1.3 `HashMap<PathBuf, TextureHandle>` 等の画像キャッシュ機構を実装する

### Definition of Done (DoD)

- [x] Markdown内の相対パス画像（PNG, JPG, GIF, SVGs）が表示されること。
- [x] 未ロード中または未発見時には適切なプレースホルダーが表示されること。
- [x] `make check-local` が exit 0 で全てパスすること。
- [x] `.agents/skills/self-review/SKILL.md` を利用して自己レビューと品質確認を行うこと。
- [ ] `.agents/skills/commit_and_push/SKILL.md` を利用してコミットとプッシュを行うこと。
- [ ] `.agents/skills/create_pull_request/SKILL.md` を利用してタスクPRを作成すること。
- [ ] `gh pr merge` 等を利用し、ベースブランチへマージして最新化すること。

---

## 2. スプラッシュスクリーン

- [ ] 2.1 起動時（eguiの初期フレーム）に約1.5秒間（初回の画面表示のロードをバックグラウンドで行うこと、メインのウィンドを非表示で開き1.5秒経過後に表示にすると複雑な制御が不要になると思われる。）、アイコン＋バージョン番号を表示する
- [ ] 2.2 フレーム推移によりフェードアウトさせるアニメーションを実装
- [ ] 2.3 画面クリックによるスプラッシュのスキップ機能を実装

### Definition of Done (DoD)

- [ ] アプリ起動時に独立したスプラッシュ画面が表示され、その後メインUIに遷移すること。
- [ ] ユーザーのクリック操作で瞬時にスキップできること。
- [ ] `make check-local` が exit 0 で全てパスすること。
- [ ] `.agents/skills/self-review/SKILL.md` を利用して自己レビューと品質確認を行うこと。
- [ ] `.agents/skills/commit_and_push/SKILL.md` を利用してコミットとプッシュを行うこと。
- [ ] `.agents/skills/create_pull_request/SKILL.md` を利用してタスクPRを作成すること。
- [ ] `gh pr merge` 等を利用し、ベースブランチへマージして最新化すること。

---

## 3. メニュー拡充

- [ ] 3.1 About ダイアログを実装し、アプリ名、バージョン、ライセンス、アイコンを表示する
- [ ] 3.2 Help メニューから GitHub リポジトリへブラウザで遷移させる
- [ ] 3.3 macOS ネイティブメニュー（`macos_menu.m`）と非macOS用フォールバックメニューに追加を適用する

### Definition of Done (DoD)

- [ ] OSネイティブのメニューから About, Help の各種ダイアログ（または遷移）が正常に呼び出せること。
- [ ] `make check-local` が exit 0 で全てパスすること。
- [ ] `.agents/skills/self-review/SKILL.md` を利用して自己レビューと品質確認を行うこと。
- [ ] `.agents/skills/commit_and_push/SKILL.md` を利用してコミットとプッシュを行うこと。
- [ ] `.agents/skills/create_pull_request/SKILL.md` を利用してタスクPRを作成すること。
- [ ] `gh pr merge` 等を利用し、ベースブランチへマージして最新化すること。

---

## 4. 画像・ダイアグラムのビューアコントロール追加

- [ ] 4.1 画像やダイアグラム（mermaid, drawio, plantuml, png, jpg, svg）の右上・右下等にオーバーレイ表示されるサブコントロールUI（ボタン群）を実装する
- [ ] 4.2 コントロールから【拡大・縮小・左右上下の移動（パン操作）・リセット等】を行えるようにする
- [ ] 4.3 コントロールのボタン（または画像ダブルクリック等）から、対象画像をモーダルで別画面領域に大きく表示する機能を追加する

### Definition of Done (DoD)

- [ ] Markdown内の画像およびダイアグラム上にコントロールUIが表示され、拡大・縮小などのパン＆ズーム操作が正常に行えること。
- [ ] モーダルでの別表示機能が正常に動作し、元のプレビュー画面全体のレイアウトに干渉したりアプリがクラッシュしないこと。
- [ ] `make check-local` が exit 0 で全てパスすること。
- [ ] `.agents/skills/self-review/SKILL.md` を利用して自己レビューと品質確認を行うこと。
- [ ] `.agents/skills/commit_and_push/SKILL.md` を利用してコミットとプッシュを行うこと。
- [ ] `.agents/skills/create_pull_request/SKILL.md` を利用してタスクPRを作成すること。
- [ ] `gh pr merge` 等を利用し、ベースブランチへマージして最新化すること。

---

## 5. 最終確認 ＆ リリース作業

> [!CAUTION]
> **【AIエージェントへの厳重警告】**
> 過去のリリースにおいて、AIが以下の致命的な違反を犯しました。
>
> 1. `make release` が失敗した際、勝手に `git tag -a` などの代替コマンドを実行し、タスクを強行完了(`[x]`)として虚偽入力した。
> 2. `openspec` のアーカイブ(`/opsx-archive`)を忘却し、不要なディレクトリを残したままリリースに進んだ。
> 3. カバレッジ低下（エラーハンドリングパスのテスト漏れ）を放置した。
> 4. `process_rules.md` で指定された「日本語での報告・コミット」を無視し英語で行った。
>
> これらはプロセスの信頼性を根底から覆す行為です。以下の手順は**絶対に独自解釈でスキップ・代替実行せず**、一つずつ確実に完了させてください。エラー発生時は即座に中断し、人間に報告すること。

- [ ] 5.1 本バージョンの `tasks.md` にて、先行するすべてのタスクが完了し `[x]` が付いていることを確認する。
- [ ] 5.2 `docs/coding-rules.ja.md` および `.agents/skills/self-review/SKILL.md` を利用して自己レビューを行う。**追加・変更したすべての実行パス（エラー処理等）がテストで網羅されているか**監査すること。
- [ ] 5.3 `make check-local` が exit 0 で完全にパスし、LLVM Coverage が 100% であることを確認する。（※エラーが出ても `--no-verify` で強行しないこと）
- [ ] 5.4 最初に作成したmasterから派生させた中間branchをmasterブランチにマージする。
- [ ] 5.5 **（重要）リリース前に必ずアーカイブを実行:** `.agents/skills/openspec-archive-change/SKILL.md` に従い、本ディレクトリ(`v0.4.0-desktop-viewer-polish`)をアーカイブ（退避・コミット）する。
- [ ] 5.6 masterに向けてPRを作成する。
- [ ] 5.7 master merge ※--adminの利用を許可。（プッシュ時にフックエラーが出た場合、独断で `--no-verify` を使わず人間へ報告すること）
- [ ] 5.8 `.agents/skills/release_workflow/SKILL.md` に従い、`make release VERSION=0.4.0` を実行し、自動タグ打ちとリリースを完了する。**（※コマンド実行に失敗した場合は絶対に代替手順を使わず、作業を即時終了してユーザーの指示を仰ぐこと。全報告・コミットは日本語厳守）**
