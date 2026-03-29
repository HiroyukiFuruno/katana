## Context

今回の change は、OpenSpec の `tasks.md` を KatanA 上で編集・確認する基本ループを崩している 2 件の UI / preview 不整合を修正するための設計です。

### Current Analysis (What)

- `crates/katana-ui/src/views/panels/preview.rs` の更新ボタンは preview ヘッダにしか存在せず、CodeOnly では操作できない
- 同ボタンが dispatch しているのは `AppAction::RefreshDiagrams` であり、`crates/katana-ui/src/app/action.rs` では texture cache の破棄と `doc.buffer` からの再描画しか行っていない
- `crates/katana-ui/src/app/preview.rs` の hash は preview 再描画のスキップ判定用であり、「ディスク上のファイル内容が前回取込時から変わったか」を表す hash としては使われていない
- `crates/katana-ui/src/app/workspace.rs::handle_refresh_workspace()` は workspace tree の再走査だけを担当し、開いている文書の buffer 再読込は行わない
- `crates/katana-ui/src/app/document.rs::handle_select_document()` は lazy-load 直後や未読込タブの活性化時だけ `fs.load_document()` を呼び、既に読み込まれたタブの再選択では既存 buffer をそのまま再利用する
- `crates/katana-core/src/document.rs::Document` は `buffer / is_dirty / is_loaded` しか保持せず、外部更新を自動判定する revision / mtime の概念がない
- 成功した初回 load / save / reload の時点で「last imported disk hash」をいつ更新するかが未定義のままだと、save 後や再起動後に false-positive な reload 判定が起こりうる
- `crates/katana-ui/src/shell_ui.rs` には auto-save の周期処理が既にあり、定期タスクを UI update loop に載せる足場はあるが、document refresh 向けの timer / status 制御は未定義
- `crates/katana-platform/src/settings/types/behavior.rs` には auto-save 設定しかなく、auto-refresh の enable / interval / default は未定義
- dirty 文書で external change が継続している場合、interval ごとに同じ warning を出すと status bar がスパム化するが、その抑止契約が未定義
- `vendor/egui_commonmark/src/parsers/pulldown.rs` では task list 項目かどうかを `task_list_indices.contains(current_event_idx)` で判定し `Tag::Item` 時に bullet を抑止している
- 一方で `item_list_wrapping()` は `delayed_events_list_item()` で回収したイベントを index なしで再列挙しており、ネストした list 解析では元の event index が保持されない
- そのため、親 item が task list であっても nested parsing 経路では task list 判定を失い、preview 上で `bullet + checkbox` の二重マーカーが発生しうる

### Required Fix Direction (How)

- 「図の再描画」と「ユーザーが明示的に外部変更を取り込む refresh」は同一 action にせず、内部 rerender とユーザー起点 reload を分離する
- shared refresh は shell 共通 chrome に置き、PreviewOnly / CodeOnly / Split で常に同じ意味で動かす
- active document ごとに「last imported disk hash」を保持し、manual / automatic refresh ともに hash 差分がある場合だけ reload 判定へ進む
- clean 文書だけを disk reload 対象にし、dirty 文書は silent overwrite せず警告 + render-only refresh に留める
- successful load / save / reload のたびに last imported disk hash を同期し、false-positive を残さない
- auto-refresh は設定可能な interval で active document だけを監視し、default 値は rationale 付きで提案してユーザー承認を得る
- dirty 文書で外部差分が見つかった場合は「external change pending」を latch し、同一差分に対する warning を毎 interval 繰り返さない
- nested task list は vendored `egui_commonmark` 側で event identity を保ちながら list item を再帰処理し、task item の bullet 抑止をネスト下でも成立させる

## Goals / Non-Goals

**Goals:**

- active document に対する共有 refresh ボタンを全 view mode で提供する
- 外部エディタで変更された clean 文書を、手動 refresh または hash 差分検知ベースの自動更新で安全に in-memory buffer へ取り込めるようにする
- dirty 文書を外部変更で黙って破壊しない契約を明文化し、UI で通知する
- 自動更新の interval と有効/無効を settings に保持し、ユーザーが変更可能にする
- nested task list 親行の二重マーカーを解消しつつ、子リストの既存表現を維持する
- UI / parser regression test を追加し、再発を防ぐ

**Non-Goals:**

- ファイル監視による自動リロードの導入
- dirty buffer と外部変更の 3-way merge UI
- workspace refresh の責務拡張（tree scan 以外の一括再読込）
- preview export / ToC / split layout 全体の再設計

## Decisions

### 1. ユーザー起点の refresh は `RefreshDiagrams` と分離する

共有更新ボタンは、新しい user-facing action（例: `RefreshDocument`）として扱い、既存の `RefreshDiagrams` は theme change や asset reload のような internal rerender 専用に残す。自動更新もこの user-facing refresh helper を再利用するが、起動条件は「hash 差分あり」のときだけに限定する。

- 採用理由:
  - 現在 `RefreshDiagrams` は theme 変更や download 完了など、ディスク再読込を伴うべきでない内部経路からも使われている
  - ユーザー起点 refresh にだけ disk reload 契約を持たせることで、既存の自動再描画経路を壊さずに機能拡張できる
- 代替案:
  - `RefreshDiagrams` の意味をそのまま拡張する: theme 変更時までディスク再読込が走り得て責務が崩れる
  - `RefreshWorkspace` に統合する: tree scan と active document reload が結合し、I/O と UX が過剰になる

### 2. refresh ボタンは `ViewModeBar` の共有 chrome に一本化する

共有 refresh ボタンは `crates/katana-ui/src/views/top_bar.rs` など shell 共通 chrome に配置し、preview pane 内の専用 refresh ボタンは撤去する。preview 側には export / ToC など preview 固有操作だけを残す。

- 採用理由:
  - CodeOnly でも同じ操作に到達できる
  - 「どの更新ボタンが disk reload なのか」という意味の分岐を消せる
- 代替案:
  - preview 内と共有 toolbar の二重配置: 同じアイコンに異なる責務が生まれやすい
  - menu 項目だけ追加する: 既存の視認性要求を満たしにくい

### 3. disk refresh は content hash を基準に判定する

active document には「最後に disk から取り込んだ内容の hash」を保持し、manual refresh では現在の on-disk hash と差分がない限り何もしない。auto-refresh も同じ hash 比較を interval ごとに実行し、差分があったときだけ reload 可否判定へ進む。preview 用 hash は render cache 用に残し、disk freshness とは分離する。

- 採用理由:
  - ユーザー要件どおり「変更がなければ更新しない」を機械的に保証できる
  - `mtime` だけでは内容未変更の save やファイルシステム差異で不要 reload が起こりうる
- 代替案:
  - `mtime` のみ比較する: 内容不変でも不要更新が走る
  - 毎回無条件に reload する: dirty 保護や render skip の意味が薄れる

### 4. disk reload は clean 文書だけに適用し、dirty 文書は保護する

共有 refresh 実行時、active document が clean なら `FilesystemService::load_document()` で最新内容を読み直して buffer を置き換える。dirty なら buffer の置換を行わず、render-only refresh と warning 表示だけを行う。

- 採用理由:
  - `Document` は外部変更との統合機構を持たず、dirty 文書を読み直すと unsaved change を無言で失う
  - clean 文書に限定すれば、外部エディタとの往復を最小実装で安全に実現できる
- 代替案:
  - 常に上書きする: データ損失リスクが高い
  - dirty 時に差分マージを行う: v0.8.6 の bugfix スコープを超える

### 5. last imported disk hash は load / save / reload 成功時に更新する

`last imported disk hash` は active document を最初に disk から開いたとき、successful save が完了したとき、successful reload が完了したときに更新する。dirty skip や read failure では更新しない。

- 採用理由:
  - save 後に古い hash が残ると、自分で保存した内容まで「外部変更」と誤認する
  - failure path で hash を進めると、未反映差分を見失う
- 代替案:
  - reload 時だけ更新する: save 後の false-positive を防げない
  - 毎 poll ごとに現在 hash へ追随する: dirty 保護の意味が失われる

### 6. auto-refresh は active document のみを polling し、設定は `behavior` に置く

定期確認は `shell_ui.rs` のメイン update loop で active document だけを対象に行う。設定は既存の `BehaviorSettings` に `auto_refresh_external_changes` と `auto_refresh_interval_secs` を追加し、Behavior タブで auto-save と同じ操作モデルで編集できるようにする。

**Proposed default:** `2.0` 秒（pending user approval）

- 理由:
  - 1 ファイル分の内容 hash を 2 秒ごとに確認するコストは desktop app では十分小さい
  - OpenSpec / Markdown の外部編集ループとしては「保存後すぐ気付ける」体感を維持しやすい
  - 既存の auto-save default `5.0` 秒より短くしても、active document 限定なら polling 密度は過剰になりにくい
- 合意条件:
  - この default はユーザー承認後に固定する
  - 承認前は design 上の提案値として扱い、実装タスクで明示確認を行う
- 代替案:
  - `1.0` 秒: 応答性は高いが polling がやや過剰
  - `5.0` 秒: 既存 auto-save と揃うが、外部エディタ反映としては遅く感じやすい

### 7. dirty 文書で検出した外部差分は pending 状態として保持し、warning を重複表示しない

auto-refresh 中に dirty 文書で external change を検出した場合、system は「external change pending」状態を保持し、同じ差分 hash に対しては warning を 1 回だけ表示する。pending は successful save、successful reload、または on-disk hash が元に戻った時点で解消する。

- 採用理由:
  - interval polling と warning 表示をそのまま結合すると UX が劣化する
  - pending 状態を持てば「外部差分あり」を失わずに通知回数だけ抑制できる
- 代替案:
  - 毎回 warning を表示する: status bar がスパム化する
  - dirty 中は検知自体を止める: 外部差分の存在を見落とす

### 8. document refresh と workspace refresh は責務を分けたままにする

workspace refresh は引き続き tree 再走査だけを行い、active document reload は shared refresh に限定する。

- 採用理由:
  - workspace refresh は複数ファイル・ディレクトリ構造の更新反映が責務であり、active tab の buffer 置換まで持たせると副作用が大きい
  - external editor で単一文書を更新したケースでは tree scan は不要
- 代替案:
  - workspace refresh 時に open docs も全再読込する: dirty 保護やタブ単位の契約が曖昧になる

### 9. nested task list 修正は vendored parser の event index 保持で行う

`delayed_events()` / `delayed_events_list_item()` / `item_list_wrapping()` 等の「一度イベントを回収してから再描画する経路」で、元の `EventIteratorItem` index を保持して `current_event_idx` を正しく更新する。これにより `task_list_indices` に基づく task item 判定を nested list でも維持する。

- 採用理由:
  - 現行の bullet 抑止ロジックを壊さず、nested path だけを正しくできる
  - custom task state（`[/]`, `[-]`, `[~]`）にも同じ判定基盤を流用できる
- 代替案:
  - `Tag::Item` 直前の lookahead だけで task list 判定をやり直す: wrapper 経路や custom marker で再び破綻しやすい
  - task list を pre-render で全部 HTML 化する: 既存の interactive checkbox と context menu を失う

### 10. 回帰防止は parser unit test と Katana UI test の二層で行う

vendor 側では nested task list の event classification を直接検証し、Katana 側では preview harness を用いた UI regression で「parent 行に余計な bullet が出ない」ことと「共有 refresh が clean / dirty で正しく分岐する」ことを担保する。

- 採用理由:
  - root cause が vendor parser にあり、症状は Katana UI に出るため、片側だけのテストでは再発を防ぎきれない
- 代替案:
  - UI test のみ: parser 層の原因切り分けが弱い
  - unit test のみ: 実際の KatanA preview レイアウト保証が弱い

## Risks / Trade-offs

- [Risk] dirty 文書で共有 refresh を押したとき、ユーザーが「外部変更が取り込まれなかった」と感じる
  - Mitigation: status bar 文言で「dirty のため disk reload をスキップした」ことを明示する

- [Risk] auto-refresh polling が無駄な read/hash を増やす
  - Mitigation: active document のみを対象にし、hash 不変時は reload を行わず、interval は設定可能にする

- [Risk] dirty 文書で同じ外部差分に対する warning が反復表示される
  - Mitigation: pending 外部差分状態を持ち、同一 hash の warning は 1 回だけに抑制する

- [Risk] vendor patch が upstream `egui_commonmark` 更新時の追従コストになる
  - Mitigation: event index 保持の変更点を局所化し、回帰テストを同梱する

- [Risk] 共有 toolbar にボタンを追加すると横幅が窮屈になる
  - Mitigation: preview 固有の refresh ボタンを撤去し、共有 toolbar のみを canonical にする

- [Risk] auto-refresh を active document のみへ限定すると、非アクティブ tab の外部変更は即時反映されない
  - Mitigation: v0.8.6 では active document の自動更新にスコープを限定し、複数 tab 監視は follow-up に分離する

## Migration Plan

1. shared refresh action と auto-refresh polling の責務を定義し、behavior settings に enable / interval を追加する
2. active document の disk hash 管理を導入し、load / save / reload 成功時の hash 更新契約を明文化する
3. clean / dirty の reload policy と external-change-pending 抑止を実装し、preview-local refresh UI を shared refresh へ置き換える
4. nested task list の vendored parser を修正し、task item の bullet suppression を nested path でも成立させる
5. parser / UI / shell / settings の regression test を追加し、Code / Preview / Split それぞれで manual verification を行う

## Open Questions

- `auto_refresh_interval_secs` の default を `2.0` 秒で確定してよいか。これは本 design の提案値であり、実装前にユーザー承認が必要。
