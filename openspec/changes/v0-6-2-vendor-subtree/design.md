## Architecture Changes

- `vendor/egui_commonmark` を削除し、外部リポジトリの特定のブランチ・タグを指す `git subtree` へと置き換えます。
- Katana 独自のパッチ（`pulldown.rs` 等におけるカスタマイズ）は、再整理して別途コミット化します。

### Components Affected

- `vendor/egui_commonmark` 配下全体

## Data Flow

特になし

## Error Handling

特になし
