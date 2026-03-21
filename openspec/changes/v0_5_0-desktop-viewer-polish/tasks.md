## Definition of Ready (DoR)

- **前提条件**: `desktop-viewer-polish-v0.4.0` の内容が `main` ブランチにマージされていること。
- **タスクの順序性**:
  - 依存はない。それぞれ独立して実装・検証可能。

## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

---

## 1. Markdown エクスポート

- [ ] 1.1 プレビュー画面のメニューに「エクスポート」のサブメニュー（HTML / PDF / PNG / JPG）を追加する
- [ ] 1.2 HTML: comrak による HTML 出力に、現在適用されているCSSスタイル（インラインまたは内部スタイルシート）を埋め込み一時ファイル化し、ブラウザで開く
- [ ] 1.3 PDF/画像: 出力された HTML を外部ツール（`wkhtmltopdf`, `weasyprint` 等）に渡し、保存先ダイアログ経由で出力する
- [ ] 1.4 外部ツールが未インストールの場合に、GUI上でエラーメッセージとインストールガイドを表示する

### Definition of Done (DoD)

- [ ] HTML, PDF, および画像ファイルへのエクスポート処理が実行され、ファイルが生成されること。
- [ ] 外部ツールが存在しない環境では、クラッシュせずに適切なエラーダイアログがユーザーに提示されること。
- [ ] `make check-local` が exit 0 で全てパスすること。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 2. 規約同意画面

- [ ] 2.1 EULA（エンドユーザー使用許諾契約）同意画面を構築し、未同意時のブロック機能を追加するi18nリソースとして組み込む
- [ ] 2.2 起動時（スプラッシュ表示後、メインUI表示前）に全画面モーダルとして規約同意UIを表示する
- [ ] 2.3 日本語・英語の切り替えプルダウンを配置する
- [ ] 2.4 「同意する」ボタンにて `AppSettings` の `terms_accepted_version` を更新・永続化する。不同意の場合はアプリを終了する
- [ ] 2.5 バージョンが上がった際に再同意を要求するロジックを実装する ※但し規約に変更がない場合は必要がないため、永続化する規約への同意の記憶はversionを含めた形で保存できる形式で実装すること。

### Definition of Done (DoD)

- [ ] 初回起動時に多言語対応の規約画面が必ず表示され、同意するまでアプリ本来の機能にアクセスできないこと。
- [ ] 一度同意すれば次回以降の起動時はスキップされること（ただし規約バージョンが上がった場合は再提示されること）。
- [ ] `make check-local` が exit 0 で全てパスすること。
- [ ] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. 検索・フィルターUIの改善 (Search Modal Options & Sidebar)

- [x] 3.1 ワークスペースサイドバーから検索モーダル（Cmd+P）を開く検索（🔍）ボタンを配置し、フィルターボタンのアイコンを漏斗（Y字）型に変更する
- [x] 3.2 検索オプション用に `AppState` へ `search_include_pattern`, `search_exclude_pattern` (String) を追加する
- [x] 3.3 検索モーダル内に含める/除外するディレクトリ・ファイルの入力欄を追加し、カンマ区切り正規表現でパースして結果をフィルタするロジックを実装する
- [x] 3.4 検索機能のITテストでTDD（REDフェーズ）を実施後、テストを通過（GREEN）させる
- [x] 3.5 新UI要素に対するi18nキー（日・英）を追加する

### Definition of Done (DoD)

- [x] TDDの原則に従い、検索オプション（Include/Exclude）に対するテストが実装・パスしていること。
- [x] 検索モーダルにInclude/Excludeオプションが存在し、カンマ区切りの正規表現で想定通りに絞り込み・除外が行えること。
- [x] ワークスペースサイドバーに検索ボタンと新しいフィルターアイコンが配置されていること。
- [x] 追加されたテキスト群が日・英のi18n対応を行っていること。
- [x] `make check-local` が exit 0 で全てパスすること。
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 4. 最終確認 ＆ リリース作業

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

- [ ] 4.1 本バージョンの `tasks.md` にて、先行するすべてのタスクが完了し `[x]` が付いていることを確認する。
- [ ] 4.2 `docs/coding-rules.ja.md` および `.agents/skills/self-review/SKILL.md` を利用して自己レビューを行う。**追加・変更したすべての実行パス（エラー処理等）がテストで網羅されているか**監査すること。
- [ ] 4.3 `make check-local` が exit 0 で完全にパスし、LLVM Coverage が 100% であることを確認する。（※エラーが出ても `--no-verify` で強行しないこと）
- [ ] 4.4 最初に作成したmasterから派生させた中間branchをmasterブランチにマージする。
- [ ] 4.5 **（重要）リリース前に必ずアーカイブを実行:** `.agents/skills/openspec-archive-change/SKILL.md` に従い、本ディレクトリ(`v0.5.0-desktop-viewer-polish`)をアーカイブ（退避・コミット）する。
- [ ] 4.6 masterに向けてPRを作成する。
- [ ] 4.7 master merge ※--adminの利用を許可。（プッシュ時にフックエラーが出た場合、独断で `--no-verify` を使わず人間へ報告すること）
- [ ] 4.8 `.agents/skills/release_workflow/SKILL.md` に従い、`make release VERSION=0.5.0` を実行し、自動タグ打ちとリリースを完了する。**（※コマンド実行に失敗した場合は絶対に代替手順を使わず、作業を即時終了してユーザーの指示を仰ぐこと。全報告・コミットは日本語厳守）**
