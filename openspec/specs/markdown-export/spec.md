## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: HTML出力

現在開いているMarkdownをHTMLファイルとしてエクスポートし、デフォルトブラウザで開く。 The system SHALL conform.

#### Scenario: HTMLエクスポート

- **WHEN** ユーザーがメニューから「HTMLとして出力」を選択する
- **THEN** Markdownが完全なHTMLファイルに変換され、一時ディレクトリに保存されてデフォルトブラウザで開かれる

#### Scenario: HTML内のスタイル

- **WHEN** HTMLがエクスポートされる
- **THEN** 現在のテーマに基づいたCSSスタイルが埋め込まれ、プレビューと同等の表示になる

### Requirement: PDF出力

現在開いているMarkdownをPDFファイルとしてエクスポートする。 The system SHALL conform.

#### Scenario: PDF出力

- **WHEN** ユーザーがメニューから「PDFとして出力」を選択する
- **THEN** 保存先ダイアログが表示され、選択した場所にPDFが保存される

#### Scenario: 外部ツール未インストール時

- **WHEN** PDF生成に必要な外部ツールがインストールされていない
- **THEN** エラーメッセージとインストールガイドが表示される

### Requirement: 画像出力（PNG / JPG）

現在開いているMarkdownをPNGまたはJPG画像としてエクスポートする。 The system SHALL conform.

#### Scenario: 画像エクスポート

- **WHEN** ユーザーがメニューから「PNGとして出力」または「JPGとして出力」を選択する
- **THEN** 保存先ダイアログが表示され、選択した場所に画像が保存される

#### Scenario: 長いドキュメントの画像出力

- **WHEN** ドキュメントがビューポートより長い
- **THEN** ドキュメント全体が1枚の縦長画像として出力される
