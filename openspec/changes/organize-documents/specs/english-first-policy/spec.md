## ADDED Requirements

### Requirement: リポジトリテキスト全般の英語記述標準化 (English language standard)
リポジトリ直下の一般ドキュメント群（`README.md`、`CONTRIBUTING.md`）、ならびにコードベース全域に対するソースコードのコメントについては、すべて英語表記（English）にて記述しなければならない（MUST）。

#### Scenario: Source code comment translation
- **WHEN** プロジェクト内に日本語記述のソースコードコメントが含まれており、それが除外対象のディレクトリ・ファイル以外である場合
- **THEN** リポジトリのOSS化の公開手続きに移る前に、すべてのソースコードコメントは適切かつ意味のある英語として翻訳されていなければならない。

### Requirement: 英語化ポリシーからの明確な除外項目 (Exclusions from English Policy)
`.gitignore` ファイルによってトラッキングが無視されている対象、および `openspec/` 以下のすべてのディレクトリ群（構成上 `specs/` や `changes/` 等トラッキングの有無を問わず）、さらに `i18n/` やその関連ファイル群は、英語化の義務から明示的に除外されなければならない（SHALL）。

#### Scenario: Existing Japanese `openspec` document
- **WHEN** 日本語で記述された新たなシステム仕様書または提案書（例： `openspec/changes` 以下のマークダウン等）が配置された
- **THEN** コントリビューターが母国語を使用して作業する利便性を損なわないため、当該 `openspec` のドキュメント群は全域がポリシー免除対象であることから、英語への翻訳を強制してはならない。
