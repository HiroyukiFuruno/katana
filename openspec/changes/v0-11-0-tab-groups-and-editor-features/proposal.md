# OpenSpec Change Proposal: Tab Groups and Session UX Improvements (v0.11.0)

## Why

KatanA には既に次の土台がある。

- tab pinning 自体は `Document.is_pinned` と `TogglePinDocument` で存在する
- workspace を開き直した際の tab restore は `workspace_tabs:{workspace_path}` persistent cache と `settings.workspace.open_tabs` fallback で既に実装されている

一方で、実利用上の不足は残っている。

- tab grouping の概念がなく、多数タブを開くと管理が難しい
- workspace session persistence は `tabs / active_idx / expanded_directories` しか持たず、pinned 状態や group 情報を保存できない
- pinning は close button、batch close、shortcut close から十分に保護されていない

したがって本 change は「last_session.json を新設する greenfield session 機能」ではなく、「既存の workspace-scoped session state を versioned に拡張し、tab groups と pin safeguards を載せる」変更として整理する。

## What Changes

- workspace-scoped `workspace_tabs` session envelope を versioned schema に拡張し、group / pinned / active / expanded state を保存できるようにする
- tab group 機能を追加し、名前・色・collapsed state を持つグループへ tab を所属させられるようにする
- tab restore 設定を追加し、workspace を開いた際に前回 tab session を復元するかをユーザーが制御できるようにする
- pinned tabs は close button、close shortcut、close all / others / left / right といった通常 close action から保護する
- pinned / grouped session の backward compatibility を保つため、旧 session payload を read 時に既定値補完で受け入れる

## Capabilities

### New Capabilities

- `editor-tab-groups`: 名前・色・collapsed state を持つ tab group

### Modified Capabilities

- `workspace-shell`: workspace-scoped session restore の強化
- `settings-persistence`: session restore setting と session schema version
- `editor-tabs`: pinned tab safeguard と grouped tab rendering

## Impact

- `crates/katana-ui/src/state/document.rs`: tab group state の導入
- `crates/katana-ui/src/views/top_bar.rs`: grouped tab rendering、context menu、pinned close safeguard
- `crates/katana-ui/src/app/action.rs`: close actions が pinned/grouped state を考慮
- `crates/katana-ui/src/app/workspace.rs`: session envelope の load/save 拡張
- `crates/katana-platform/src/settings/types/workspace.rs` または related settings type: restore setting の追加
- `crates/katana-ui/locales/*.json`: group / pin safeguard / restore setting 文言
