## ADDED Requirements

### Requirement: 肥大化ファイルの責務分離

テスト除外で200行を超える全ファイルを、責務単位でサブモジュールに分割する (MUST be separated).

#### Scenario: Verify file decomposition
- **Given** a file with more than 200 lines excluding tests
- **When** the code is refactored
- **Then** the file is split into submodules by responsibility

#### katana-ui（最重要: 15ファイルが200行超）

| 対象ファイル | 現在の行数 | 分割先 |
|---|---|---|
| `shell_ui.rs` | 5,118 | `ui/menu/`, `ui/workspace/`, `ui/split/`, `ui/modals/` 等 |
| `shell.rs` | 3,144 | `app/` サブモジュール群 |
| `preview_pane.rs` | 1,816 | `preview/` サブモジュール群 |
| `settings_window.rs` | 1,666 | `settings/` タブごと |
| `preview_pane_ui.rs` | 1,270 | `preview/` に統合 |
| `i18n.rs` | 1,092 | `i18n/` サブモジュール |
| `widgets.rs` | 948 | `widgets/` コンポーネントごと |
| `font_loader.rs` | 838 | 必要に応じて分割 |
| `app_state.rs` | 795 | `state/` サブモジュール群 |
| `svg_loader.rs` | 795 | 分割 |
| `http_cache_loader.rs` | 786 | 分割 |
| `html_renderer.rs` | 635 | 分割 |
| `main.rs` | 595 | `setup/` サブモジュール |
| `changelog.rs` | 515 | 分割 |
| `about_info.rs` | 335 | 分割 |

katana-ui の対象ファイルは、単なる `render_*` free function の別ファイル移設ではなく、Reactコンポーネント相当の component 境界に沿って分割する。

#### katana-core（7ファイルが200行超）

| 対象ファイル | 現在の行数 | 分割先 |
|---|---|---|
| `html/parser.rs` | 676 | `html/parser/` サブモジュール |
| `update.rs` | 646 | `update/` サブモジュール |
| `markdown/color_preset.rs` | 495 | `color_preset/dark.rs + light.rs` |
| `preview.rs` | 472 | `preview/` サブモジュール |
| `markdown/drawio_renderer.rs` | 408 | 内部分割 |
| `html/node.rs` | 339 | `types.rs + impls.rs` |
| `markdown/export.rs` | 316 | 分割 |

#### katana-platform（7ファイルが200行超）

| 対象ファイル | 現在の行数 | 分割先 |
|---|---|---|
| `settings.rs` | 653 | `settings/` 完全移行 |
| `theme/builder.rs` | 473 | `builder/` サブモジュール |
| `cache.rs` | 291 | `cache/` サブモジュール |
| `filesystem.rs` | 279 | 分割 |
| `theme/migration.rs` | 262 | 分割 |
| `settings/types.rs` | 256 | `types/` サブモジュール |
| `theme/types.rs` | 241 | 分割 |

#### katana-linter（4ファイルが200行超）

| 対象ファイル | 現在の行数 | 分割先 |
|---|---|---|
| `rules/rust.rs` | 969 | `rules/rust/` Visitorごと |
| `rules/locales.rs` | 549 | 分割 |
| `utils.rs` | 406 | `utils/` サブモジュール |
| `rules/i18n.rs` | 391 | 分割 |
