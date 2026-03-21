# Local Asset Preview

## Purpose
TBD (Displaying local image assets in markdown preview)

## Requirements

### Requirement: Markdown内ローカル画像の遅延読み込み表示
Markdown内の相対パスで参照されたローカル画像ファイルを、プレビューで表示する。初期表示を遅くしないため遅延読み込み（lazy load）とする。

#### Scenario: 相対パス画像の表示
- **WHEN** Markdownに `![alt](./images/photo.png)` のようなローカル相対パス画像参照がある
- **THEN** プレビューでその画像がインラインで表示される

#### Scenario: 遅延読み込みによる初期表示パフォーマンス維持
- **WHEN** 多数の画像を含むMarkdownファイルを開く
- **THEN** テキスト部分は即座に表示され、画像はビューポートに入った時点で読み込まれる

#### Scenario: 画像ファイルが存在しない場合
- **WHEN** 参照された相対パスの画像ファイルが存在しない
- **THEN** プレースホルダー（altテキスト + 不明アイコン）が表示される

#### Scenario: 対応画像フォーマット
- **WHEN** PNG, JPG, GIF, SVG のいずれかの画像が参照される
- **THEN** 全てのフォーマットが正しく表示される
