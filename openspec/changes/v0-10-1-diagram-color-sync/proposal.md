## Why

現在の diagram rendering は、theme change 後の preview text color と完全には同期していない。`shell_ui.rs` では `ThemeColors` を `egui::Context` に保存し、theme change 時に `RefreshDiagrams` を発火しているが、Mermaid/PlantUML 系の renderer は依然として `DiagramColorPreset::current()` と dark/light の 2 値に依存している。そのため、同じ dark mode 内で `preview.text` などの実色を変えても、diagram cache key と renderer 入力が古いまま残り、再描画後も色が追従しない。

問題の本質は `OnceLock` そのものではなく、「renderer が render request ごとの theme snapshot を受け取っていないこと」と「cache key が dark/light 2 値にしか反応していないこと」である。

## What Changes

- **diagram theme を request-scoped にする**: Mermaid / PlantUML / 同じ theme helper を使う diagram backend へ、現在の `ThemeColors` から導いた dynamic theme snapshot を render request ごとに渡す。
- **global preset 依存を render path から外す**: `DiagramColorPreset::current()` と global dark flag を diagram render の主要経路では使わず、explicit theme parameter に置き換える。
- **cache key に theme fingerprint を含める**: diagram cache は dark/light だけでなく、実際の text/background/theme 情報に追従して invalidation されるようにする。
- **theme change 後の refresh を実色連動にする**: 既存の `RefreshDiagrams` action は維持するが、再描画結果が現在の preview/theme color と一致するようにする。
- **legacy static preset は必要最小限に残す**: static dark/light preset を全廃するのではなく、非 UI path や fallback 用に必要な箇所だけに限定する。

## Capabilities

### Modified Capabilities

- `diagram-block-preview`: diagram preview は現在の theme snapshot を使って描画される
- `theme-settings`: theme / preview text color の変更は restart なしで diagram preview に反映される

## Impact

- `crates/katana-ui/src/shell_ui.rs`: theme change 時の diagram refresh 起点は維持する
- `crates/katana-ui/src/preview_pane/core_render.rs`: render job に theme snapshot を載せる
- `crates/katana-ui/src/preview_pane/renderer.rs`: dispatch と cache key を theme-aware にする
- `crates/katana-core/src/markdown/mermaid_renderer/render.rs`: current preset 参照を explicit theme parameter へ置き換える
- `crates/katana-core/src/markdown/plantuml_renderer.rs`: current preset 参照を explicit theme parameter へ置き換える
- `crates/katana-core/src/markdown/drawio_renderer/*`: 同一 helper を使う経路がある場合は同じ theme snapshot を共有する
