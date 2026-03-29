## Context

現在のワークスペース UI は 2 箇所に責務が分かれている。

- `crates/katana-ui/src/views/app_frame.rs`
  - `show_workspace = false` のとき、`workspace_collapsed` として単一の `ChevronRight` トグル列だけを描画する
- `crates/katana-ui/src/views/panels/workspace.rs`
  - ペイン表示時にタイトル文字列、履歴、更新、フィルター、検索、全展開、全閉を同じヘッダー周辺に描画する

現状解析で確定している事実は以下。

- 検索モーダル起動は `state.layout.show_search_modal = true` に集約されており、ショートカット (`Cmd+P`) とボタン導線は同じ state を使える
- 最近のワークスペース履歴は `settings.workspace.paths` を表示しており、`OpenWorkspace` / `RemoveWorkspace` action が既にある
- ワークスペース表示有無は `state.layout.show_workspace` の bool で完結しており、追加の layout state がなくてもレール化は可能
- 現状の collapsed 状態では、検索と履歴の導線が消える

今回の change の本質は、新しい機能を増やすことではなく、既存 action と state を左レールへ再配置して UX を整理することにある。

## Goals / Non-Goals

**Goals:**

- ワークスペース表示切り替え・検索・履歴を、ペインの開閉と独立した左アクティビティレールへ移す
- ワークスペースペインのヘッダーからタイトル文字列を除去し、操作とツリー表示を優先する
- 更新/フィルターと全展開/全閉を 2 グループに分け、役割を判別しやすくする
- 既存 icon / action / state を最大限再利用し、新しい永続化や複雑な UI state を増やさない
- 他の実装者が会話履歴なしで読んでも、変更対象 file と UI state が分かる状態にする

**Non-Goals:**

- ワークスペースツリーのノード表示仕様や検索アルゴリズムの変更
- 新アイコン資産の導入
- 履歴保存形式や検索ショートカット仕様の変更
- フィルターの正規表現仕様や計算ロジックの変更

## Target State

この change 完了時点でのあるべき状態は次のとおり。

- 左端に常時表示の細いアクティビティレールがある
- `show_workspace = false` でも、ワークスペース再表示・検索・履歴の導線が残る
- ワークスペースヘッダーには `Workspace` / `ワークスペース` の文字列がない
- ヘッダー先頭側に更新/フィルター、末尾側に全展開/全閉がある
- 検索はショートカットと左レールの両方から同じ `show_search_modal` を開く
- 履歴は `settings.workspace.paths` をそのまま使い、新しい persistence を持たない
- 実装途中でレイアウト制約が設計とずれた場合、先に OpenSpec artifact を更新してからコードを進める

## Decisions

### 1. `workspace_collapsed` を専用アクティビティレールへ置き換える

`app_frame.rs` にある collapsed 用の単一トグル列は廃止し、常時表示の細い左レールへ置き換える。これにより、ペインを閉じても検索と履歴の導線が残る。

- 採用理由:
  - 現行の最大の UX 欠点は「閉じると頻出導線が消える」こと
  - `show_workspace` bool は既に存在し、レール常駐化に追加 state を要しない
- 代替案:
  - ペインヘッダー内の再配置だけで済ませる
  - これは collapsed 時の導線欠落を解消できないため不採用

### 2. レールは既存 action/state へ直接つなぐ

レール上の 3 ボタンは新しい action を増やさず、既存経路へ接続する。

- ワークスペース表示切り替え
  - `state.layout.show_workspace` を toggle
- 検索
  - `state.layout.show_search_modal = true`
- 履歴
  - `settings.workspace.paths` を使ってメニューを描画し、`OpenWorkspace` / `RemoveWorkspace` を dispatch

これにより、ロジック変更は最小化し、主な変更対象を UI 配置へ限定する。

### 3. ワークスペースヘッダーは「現ワークスペース固有操作」だけに絞る

ヘッダーからタイトル文字列、検索、履歴を外し、残す操作を 2 グループへ分離する。

- 先頭側グループ
  - 更新
  - フィルター
- 末尾側グループ
  - 全展開
  - 全閉

フィルター入力欄は現行どおりトグル直下に出し、`filter_enabled` / `filter_query` / `filter_cache` をそのまま使う。

### 4. 履歴ボタンは 0 件でも非活性で残す

履歴が 0 件でもボタンは配置したまま disabled にする。理由はレールのアイコン並びを固定し、muscle memory を崩さないため。

### 5. 実装 blueprint: 変更対象と責務

最低限の責務分割を以下で固定する。

- `crates/katana-ui/src/views/app_frame.rs`
  - 左アクティビティレールの枠を描画
  - `show_workspace` の true/false に応じて、ペイン本体の有無だけを切り替える
- `crates/katana-ui/src/views/panels/workspace.rs`
  - タイトル文字列を除去
  - ヘッダーのボタン群を 2 グループへ再配置
  - 検索/履歴 UI を削除
- `crates/katana-ui/src/state/layout.rs`
  - `show_workspace` / `show_search_modal` を既存のまま使う
- `crates/katana-ui/locales/*.json`
  - レールの hover text が既存文言で不足する場合のみ追加

実装順序は次を推奨する。

1. `app_frame.rs` の collapsed 列をレールへ置き換える
2. `workspace.rs` ヘッダーから検索/履歴を外し、2 グループへ再配置する
3. レールの検索/履歴ボタンを既存 state/action に接続する
4. no-workspace / workspace-open / workspace-collapsed の 3 状態を回帰確認する

### 6. 前提が崩れた場合は artifact を先に更新する

以下の条件では、実装者は先に artifact を更新してから次へ進む。

- `app_frame.rs` の panel 構造上、レールを別 panel に分離しないと安定しないと分かった場合
- 履歴 0 件で disabled 表示よりも非表示の方が妥当だと判断できる具体的理由が出た場合
- ヘッダー 2 グループ化でフィルター入力欄の現位置維持が困難と分かった場合

是正フロー:

1. 制約や試作結果を `design.md` に追記する
2. 影響する requirement を `specs/*/spec.md` で修正する
3. 実装順序や検証項目が変わるなら `tasks.md` を更新する
4. その後にコード実装へ戻る

## Risks / Trade-offs

- **[Risk] 左レール追加で横幅が減る**
  - Mitigation: 幅は collapsed 列と同程度の固定最小幅に抑え、タイトル文字列削除で相殺する
- **[Risk] `ChevronLeft/Right` は意味が直感的でない**
  - Mitigation: active fill と tooltip で現在状態を補う
- **[Risk] レールとヘッダーに責務が割れて導線が曖昧になる**
  - Mitigation: レールは主要導線、ヘッダーは現ワークスペース固有操作、と責務を固定する

## Migration Plan

- 永続化形式変更は行わない
- 既存ショートカット、履歴更新順、ワークスペース action は維持する
- UI 導線のみを差し替える
