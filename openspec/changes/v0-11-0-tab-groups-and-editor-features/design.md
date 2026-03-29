## Context

現状の tab/session 関連コードには、既に使える土台と不足点が混在している。

- `crates/katana-core/src/document.rs`
  - `Document` は `is_pinned` を持つ
- `crates/katana-ui/src/app/action.rs`
  - `TogglePinDocument` は pin/unpin と pinned-first stable sort を実装済み
  - しかし `CloseDocument`, `CloseOtherDocuments`, `CloseAllDocuments`, `CloseDocumentsToRight`, `CloseDocumentsToLeft` は pin 状態を考慮しない
- `crates/katana-ui/src/views/top_bar.rs`
  - pin 状態は title の `📌` と context menu の pin/unpin で扱われる
  - close button は pinned tab にも表示される
- `crates/katana-ui/src/app/workspace.rs`
  - `workspace_tabs:{workspace_path}` cache に `tabs`, `active_idx`, `expanded_directories` を保存している
  - 起動後は `last_workspace` 復元から `workspace_tabs` を読み、tabs を開き直している
- `crates/katana-platform/src/settings/types/workspace.rs`
  - `open_tabs`, `active_tab_idx`, `last_workspace`, `paths` を保持している

つまり、proposal にあった「`last_session.json` を新設して session restore を作る」は現状に合っていない。既存の workspace-scoped restore を捨てて別経路を生やすより、今ある `workspace_tabs` payload を versioned session envelope へ育てる方が自然である。

## Goals / Non-Goals

**Goals:**

- workspace-scoped session state を拡張し、group / pinned / active / expanded state を一貫して保存・復元できるようにする
- tab groups を導入し、name / color / collapsed state を UI から操作できるようにする
- pinned tabs を通常の close action 群から保護する
- restore behavior を setting で ON/OFF できるようにする
- 他の実装者が会話履歴なしで読んでも、保存モデル・rendering model・close policy を判断できるようにする

**Non-Goals:**

- 新しい global session file (`last_session.json`) の導入
- workspace を跨いだ global tab group 共有
- multiple instances 間での live session sync
- tab drag-and-drop UX の全面再設計

## Target State

この change 完了時点で目指す状態は次のとおり。

- workspace ごとに versioned session envelope が保存される
- session envelope は `open tab order`, `active tab`, `expanded directories`, `pinned`, `tab groups` を持つ
- tab groups は path ベース membership を持ち、1 tab は高々 1 group に所属する
- pinned tab は group に所属しない
- grouped tab UI は open document order を土台に描画され、group block の位置は「最初の member tab」の位置で安定する
- collapsed group は member tab を非表示にするが、active member が存在する場合はその 1 件だけ見える
- pinned tab は close button / shortcut / batch close で閉じられず、明示的 unpin 後のみ閉じられる
- restore setting が OFF の場合、workspace を開いても saved tab session は自動適用されない
- 旧 session payload からの read は既定値補完で成立し、新 payload は version を持つ

## Decisions

### 1. 新しい `last_session.json` は作らず、既存 `workspace_tabs` session を拡張する

session restore は既に workspace-scoped に存在するため、新しい global session file を追加すると経路が二重化する。v0.11.0 では `workspace_tabs:{workspace_path}` payload を versioned に拡張する。

- 採用理由:
  - 既存の workspace restore と自然に整合する
  - group/pin が workspace 境界に従う
  - global session file の競合や移行が増えない
- 代替案:
  - `last_session.json` 新設: 既存復元経路と競合するため不採用

### 2. session envelope は versioned schema とし、旧 payload を read 時に昇格させる

現行 payload は無名 struct の `tabs`, `active_idx`, `expanded_directories` だけである。新 schema は version を明示する。

想定する最小形:

```rust
struct WorkspaceTabSessionV2 {
    version: u32,
    tabs: Vec<WorkspaceTabEntry>,
    active_path: Option<String>,
    expanded_directories: HashSet<String>,
    groups: Vec<TabGroup>,
}

struct WorkspaceTabEntry {
    path: String,
    pinned: bool,
}

struct TabGroup {
    id: String,
    name: String,
    color_hex: String,
    collapsed: bool,
    members: Vec<String>,
}
```

旧 payload 読み込み時は次で昇格する。

- `version` が無ければ V1 legacy とみなす
- `tabs: Vec<String>` は `pinned = false` の `WorkspaceTabEntry` へ変換する
- `groups` は空配列で補完する
- `active_idx` は `active_path` へ変換できる場合のみ変換する

### 3. group membership は tab index ではなく document path で持つ

runtime index は reorder や close で変動する。group persistence は path で持つ。

- 1 tab は高々 1 group に所属
- group block の表示位置は、`open_documents` 順で最初に現れる member path に anchored する
- rendering 時は「この group をまだ描いたか」を見て block 単位で描画する

追加の相互作用ルールとして、pinned tab は group に所属しないものとする。

- pin された tab を group へ追加する UI は出さない、または無効にする
- group 所属 tab を pin した場合は、先にその group membership を外す

これにより、pinned-first ordering と grouped block ordering の二重ルールを避ける。

この設計なら canonical order は `open_documents` のままで済み、group 導入で state の正規順序を二重管理しなくてよい。

### 4. collapsed group は UI 表示だけを畳み、tab 自体は閉じない

collapsed group は session / runtime の open tab を消さず、tab bar 上で member を隠すだけにする。これにより restore・preview cache・dirty state との整合を壊さない。

ただし active document がその group に属している場合、active member だけは header の右隣に残す。そうしないと「現在編集中の tab が tab bar から消える」ため、選択状態と可視状態が乖離してしまう。

### 5. pinned safeguard は「通常 close action から閉じられない」で統一する

保護対象は 1 箇所ではなく close policy 全体で定義する。

保護対象:

- tab close button
- `CloseDocument`
- `CloseAllDocuments`
- `CloseOtherDocuments`
- `CloseDocumentsToRight`
- `CloseDocumentsToLeft`
- close shortcut から dispatch される close action

許可される close:

- unpin 後の通常 close
- 必要なら internal force path だが、UI からは直接到達させない

### 6. restore setting は workspace/session behavior に属する

追加 setting は `WorkspaceSettings` もしくは同等の workspace/session 設定領域へ置く。意味は「workspace を開いた時に saved tab session を自動適用するか」であり、editor 一般設定ではない。

default は ON とするが、`#[serde(default)]` で既存 settings との互換を保つ。

### 7. 実装 blueprint を file/unit 単位で固定する

- `crates/katana-ui/src/state/document.rs`
  - runtime tab group state を保持する
- `crates/katana-ui/src/views/top_bar.rs`
  - group header / group member / pinned close button 制御 / context menu
- `crates/katana-ui/src/app/action.rs`
  - close actions が pinned tabs を除外する
  - group add/remove/collapse/color/name edit action が必要なら追加する
- `crates/katana-ui/src/app/workspace.rs`
  - session envelope の load/save
  - legacy payload の read-time upgrade
- `crates/katana-platform/src/settings/types/workspace.rs`
  - restore setting の追加
- `crates/katana-ui/locales/*.json`
  - group / pin / restore setting 文言

### 8. 前提が崩れた場合は artifact を先に是正する

以下が判明した場合、実装者は先に OpenSpec artifact を更新する。

- group rendering が `open_documents` 順アンカーでは成立せず、別の canonical order が必要になった場合
- workspace-scoped session では足りず、global session 概念が本当に必要だと分かった場合
- pinned safeguard の対象 action がさらに広く、UI close policy の再定義が必要になった場合

是正フロー:

1. `design.md` に実測・試作結果を追記する
2. 影響する requirement を `specs/*.md` で修正する
3. 実装順・検証項目が変わるなら `tasks.md` を更新する
4. その後にコードへ戻る

## Risks / Trade-offs

- **[Risk] group rendering が tab bar code を複雑にする**
  - Mitigation: canonical order は `open_documents` に固定し、group block だけを projection layer で表現する
- **[Risk] legacy session payload 互換を壊す**
  - Mitigation: version field 導入と read-time upgrade で旧 payload を受ける
- **[Risk] pinned safeguard が batch close UX と衝突する**
  - Mitigation: batch close は「pinned を飛ばして close」へ統一し、何が残るかを predictable にする
