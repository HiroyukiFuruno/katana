## Definition of Ready (DoR)

- **前提条件**: `desktop-viewer-polish-v0.3.0` の内容が `main` ブランチにマージされていること。
- **タスクの順序性**:
  - `タスク4 (画像・ダイアグラムのビューアコントロール追加)` は `タスク1 (ローカル画像のプレビュー)` が完了し、画像レンダリング基盤が整ってから着手すること。

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. ローカル画像の遅延読み込みプレビュー

- [x] 1.1 comrak AST から画像ノードのローカル相対パスを検出・解決する
- [x] 1.2 バックグラウンドで全画像を読み込み、完了するまでプレースホルダーを表示する
- [x] 1.3 `HashMap<PathBuf, TextureHandle>` 等の画像キャッシュ機構を実装する

### Definition of Done (DoD)

- [x] Markdown内の相対パス画像（PNG, JPG, GIF, SVGs）が表示されること。
- [x] 未ロード中または未発見時には適切なプレースホルダーが表示されること。
- [x] `make check-local` が exit 0 で全てパスすること。
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. スプラッシュスクリーン

- [x] 2.1 起動時（eguiの初期フレーム）に約1.5秒間（初回の画面表示のロードをバックグラウンドで行うこと、メインのウィンドを非表示で開き1.5秒経過後に表示にすると複雑な制御が不要になると思われる。）、アイコン＋バージョン番号を表示する
- [x] 2.2 フレーム推移によりフェードアウトさせるアニメーションを実装
- [x] 2.3 画面クリックによるスプラッシュのスキップ機能を実装
- [x] 2.4 スプラッシュ画面を画面中央に配置し、読み込み状態とフェイク・テキストを含むプログレスバーを実装

### Definition of Done (DoD)

- [x] アプリ起動時に独立したスプラッシュ画面が表示され、その後メインUIに遷移すること。
- [x] ユーザーのクリック操作で瞬時にスキップできること。
- [x] `make check-local` が exit 0 で全てパスすること。
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. メニュー拡充および更新確認機能（FB反映）

- [x] 3.1 About ダイアログのi18n化（ハードコード文字列の排除）
- [x] 3.2 メインメニュー全体の構造見直しとi18n化（macOSネイティブメニューおよびシェルUI）
- [x] 3.3 GitHubリポジトリのRelease APIを叩き、現在のバージョンと比較する `Check for Updates` (更新を確認) 機能の追加
- [x] 3.4 起動時の1.5秒待機を利用した並列での自動バージョン確認および起動時通知（新しいバージョンが見つかった場合に `brew upgrade katana` コマンドを促す案内ダイアログの表示）

### Definition of Done (DoD)

- [x] About, Help, メインメニューの各項目が正しくi18nで切り替わること。
- [x] OSネイティブのメニューまたはシェルUIから機能が正常に呼び出せること。
- [x] アプリ起動時および「更新を確認」実行時に最新バージョンの判定が行われ、対象時には適切な更新ダイアログが表示されること。
- [x] `make check-local` が exit 0 で全てパスすること。
- [x] `/openspec-delivery` を実行してデリバリを完了させること。

---

## 4. 画像・ダイアグラムのビューアコントロール追加

- [x] 4.1 画像やダイアグラム（mermaid, drawio, plantuml, png, jpg, svg）の右上・右下等にオーバーレイ表示されるサブコントロールUI（ボタン群）を実装する
- [x] 4.2 コントロールから【拡大・縮小・左右上下の移動（パン操作）・リセット等】を行えるようにする
- [x] 4.3 コントロールのボタン（または画像ダブルクリック等）から、対象画像をモーダルで別画面領域に大きく表示する機能を追加する

### Definition of Done (DoD)

- [x] Markdown内の画像およびダイアグラム上にコントロールUIが表示され、拡大・縮小などのパン＆ズーム操作が正常に行えること。
- [x] モーダルでの別表示機能が正常に動作し、元のプレビュー画面全体のレイアウトに干渉したりアプリがクラッシュしないこと。
- [x] `make check-local` が exit 0 で全てパスすること。
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

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

- [x] 5.1 本バージョンの `tasks.md` にて、先行するすべてのタスクが完了し `[x]` が付いていることを確認する。
- [x] 5.2 `docs/coding-rules.ja.md` および `.agents/skills/self-review/SKILL.md` を利用して自己レビューを行う。**追加・変更したすべての実行パス（エラー処理等）がテストで網羅されているか**監査すること。
- [x] 5.3 `make check-local` が exit 0 で完全にパスし、LLVM Coverage が 100% であることを確認する。（※エラーが出ても `--no-verify` で強行しないこと）
- [ ] 5.4 最初に作成したmasterから派生させた中間branchをmasterブランチにマージする。
- [ ] 5.5 **（重要）リリース前に必ずアーカイブを実行:** `.agents/skills/openspec-archive-change/SKILL.md` に従い、本ディレクトリ(`v0.4.0-desktop-viewer-polish`)をアーカイブ（退避・コミット）する。
- [ ] 5.6 masterに向けてPRを作成する。
- [ ] 5.7 master merge ※--adminの利用を許可。（プッシュ時にフックエラーが出た場合、独断で `--no-verify` を使わず人間へ報告すること）
- [ ] 5.8 `.agents/skills/release_workflow/SKILL.md` に従い、`make release VERSION=0.4.0` を実行し、自動タグ打ちとリリースを完了する。**（※コマンド実行に失敗した場合は絶対に代替手順を使わず、作業を即時終了してユーザーの指示を仰ぐこと。全報告・コミットは日本語厳守）**
