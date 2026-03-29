## Context

現状の theme change 経路は次のようになっている。

- `crates/katana-ui/src/shell_ui.rs`
  - `ThemeColors` を `egui::Context` の temp data に保存する
  - dark/light mode を `DiagramColorPreset::set_dark_mode(dark)` へ反映する
  - theme change 時に `AppAction::RefreshDiagrams` を発火する
- `crates/katana-ui/src/preview_pane/renderer.rs`
  - diagram cache key は `md_file_path`, `kind`, `source`, `DiagramColorPreset::is_dark_mode()` で決まる
- `crates/katana-core/src/markdown/mermaid_renderer/render.rs`
  - `run_mmdc_process()` が `DiagramColorPreset::current()` を直接読む
- `crates/katana-core/src/markdown/plantuml_renderer.rs`
  - `run_plantuml_process()` が `DiagramColorPreset::current()` を直接読む

一方で、preview text や MathJax は既に `ThemeColors` 由来の実色を使っており、`crates/katana-ui/src/preview_pane/math.rs` では cache key に exact text color を含めている。diagram だけが dark/light の 2 値に留まり、custom preview text color や同一 mode 内の theme variation を拾えていない。

したがって本件の本質は、diagram renderer が render request ごとの theme snapshot を受け取っていないことと、cache key が theme variation を識別できていないことにある。

## Goals / Non-Goals

**Goals:**

- theme / preview text color 変更後の diagram preview が restart なしで現在色へ追従する
- Mermaid / PlantUML を最低限の必須対象とし、同一 helper を使う diagram backend も theme source を統一する
- diagram cache key が dark/light だけでなく current theme fingerprint に反応する
- 他の実装者が、どこで theme を解決し、どこで cache key を変えるべきかを読み取れる状態にする

**Non-Goals:**

- 設定 UI 自体の redesign
- `ThemeColors` 全体を diagram backend ごとにフル直列化して保存すること
- `DiagramColorPreset` の static preset 定義を完全削除すること

## Target State

この change 完了時点でのあるべき状態は次のとおり。

- diagram render request は request-scoped な `DiagramRenderTheme` を持つ
- renderer は global `DiagramColorPreset::current()` を主要 render path で参照しない
- `RefreshDiagrams` 後の再描画結果は current `ThemeColors` と一致する
- diagram cache key は source だけでなく theme fingerprint でも分岐する
- 同じ dark mode 内で `preview.text` や関連 diagram color が変わった場合も cache miss になり、再描画される
- 前提が崩れた場合は、コードを進める前に design/spec/tasks を更新する

## Decisions

### 1. diagram theme は `ThemeColors` から request ごとに生成する

diagram backend へ渡す theme 情報は、global state ではなく `ThemeColors` から都度導いた owned snapshot とする。ここでは仮に `DiagramRenderTheme` と呼ぶ。

最低限必要な項目:

- `background`
- `text`
- `stroke`
- `fill`
- `arrow`
- `mermaid_theme`
- `plantuml_class_bg`
- `plantuml_note_bg`
- `plantuml_note_text`
- `theme_fingerprint`

`theme_fingerprint` は cache key と test assertion に使う安定値であり、preview.text と diagram 関連色が変われば変化しなければならない。

### 2. theme snapshot は UI thread で解決し、worker thread へ載せる

renderer worker thread で `egui::Context` や global preset を読みに行かず、UI thread 側で current `ThemeColors` から `DiagramRenderTheme` を生成して render job に含める。

- 採用理由:
  - worker 側の global 依存を減らせる
  - render request と cache key を同じ theme snapshot で揃えられる
- 変更対象:
  - `crates/katana-ui/src/preview_pane/core_render.rs`
  - `crates/katana-ui/src/preview_pane/renderer.rs`

### 3. Mermaid / PlantUML は explicit theme parameter を受ける

`render_mermaid` / `run_mmdc_process`、`render_plantuml` / `run_plantuml_process` は `DiagramColorPreset::current()` を直接読まず、`DiagramRenderTheme` または等価の parameter を受ける。

この change では、`OnceLock` の static preset 定義自体を問題視しない。問題なのは render path の global lookup であり、そこだけを切り離す。

### 4. cache key は dark/light ではなく theme fingerprint を使う

`crates/katana-ui/src/preview_pane/renderer.rs` の cache key は、現在の `is_dark_mode()` bool ではなく request-scoped な `theme_fingerprint` を含む。

これにより、同一 dark mode 内で `preview.text` が変わった場合も stale cache を再利用しない。

### 5. Draw.io など共通 helper 利用 backend も theme source を揃える

今回のユーザー報告は Mermaid / PlantUML が中心だが、`DiagramColorPreset::current()` に依存する diagram backend を部分的に残すと、次のデグレを招く。したがって、diagram theme helper を共有する backend は同じ request-scoped theme source を参照する。

ただし、必須回帰テスト対象は Mermaid / PlantUML を優先する。

### 6. 実装 blueprint: 変更対象と責務

最低限の責務分割を以下で固定する。

- `crates/katana-ui/src/theme_bridge/mod.rs`
  - `ThemeColors` から `DiagramRenderTheme` を生成する helper を追加する
- `crates/katana-ui/src/preview_pane/core_render.rs`
  - render job に `DiagramRenderTheme` を持たせる
- `crates/katana-ui/src/preview_pane/renderer.rs`
  - `get_cache_key` に `theme_fingerprint` を含める
  - `dispatch_renderer` へ theme parameter を渡す
- `crates/katana-core/src/markdown/mermaid_renderer/render.rs`
  - `run_mmdc_process` が explicit theme を使う
- `crates/katana-core/src/markdown/plantuml_renderer.rs`
  - `inject_theme` / `run_plantuml_process` が explicit theme を使う

実装の最小イメージ:

```rust
struct DiagramRenderTheme {
    background: String,
    text: String,
    stroke: String,
    fill: String,
    arrow: String,
    mermaid_theme: String,
    plantuml_class_bg: String,
    plantuml_note_bg: String,
    plantuml_note_text: String,
    theme_fingerprint: String,
}
```

### 7. 前提が崩れた場合は artifact を先に更新する

以下の条件では、実装者は先に artifact を更新してから次へ進む。

- `ThemeColors` から diagram 用 color を一意に決められず、追加の theme mapping rule が必要と分かった場合
- theme fingerprint が cache invalidation に過不足を起こすと分かった場合
- Mermaid / PlantUML / Draw.io で共通 helper を使えず、backend ごとの theme struct 分離が必要と分かった場合

是正フロー:

1. 実測または試作結果を `design.md` に追記する
2. 影響する requirement を `specs/*/spec.md` で修正する
3. 実装順序や検証項目が変わるなら `tasks.md` を更新する
4. その後にコード実装へ戻る

## Risks / Trade-offs

- **[Risk] theme snapshot が大きくなりすぎる**
  - Mitigation: backend が必要とする色だけへ絞り、fingerprint もその subset から導く
- **[Risk] cache key を theme-aware にすると cache hit 率が下がる**
  - Mitigation: stale 色を再利用するより正しさを優先し、必要最小限の color set だけ fingerprint に含める
- **[Risk] static preset と dynamic theme が二重管理に見える**
  - Mitigation: static preset は fallback / non-UI path 用、dynamic theme は preview render path 用と責務を分ける
