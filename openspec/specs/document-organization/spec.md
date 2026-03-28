## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: `docs/` ディレクトリ配下からの計画・仕様の撤去と明確化
`docs/` ディレクトリは、当リポジトリおよびプロジェクトの現在の利用方法および仕組みを説明するドキュメントのみを配置する場所としなければならない（MUST）。将来のロードマップや実装前の仕様（openspec等）は、すべて `docs/` ディレクトリから削除または移動させなければならない。

#### Scenario: Contributor adds a new planning document to `docs/`
- **WHEN** コントリビューターが将来の開発構想や仕様検討用のドキュメントを `docs/` 配下に新設する Pull Request を作成した
- **THEN** その追加は拒否され、コントリビューターに対して `openspec` として別途管理するよう指示が出される。

### Requirement: `docs` およびルート `README.md` における2言語サポート体制 (Dual-language support)
`docs/` ディレクトリに含まれる全ドキュメント、およびプロジェクトルートの `README.md` は、メインである英語版と並行して、国内ユーザー向けの日本語版（Japanese version）を必ず維持管理しなければならない（MUST）。また、これらの日本語ドキュメントの命名は必ず元のファイル名に `.ja` を付与した形式（例： `*.ja.md`）に限定しなければならない（MUST）。

#### Scenario: Updating existing `docs/` content
- **WHEN** 作成者が `docs/README.md` または プロジェクトルートの `README.md` に更新を加えた
- **THEN** 作成者は同様に `docs/README.ja.md` やプロジェクトルートの `README.ja.md` の内容も同期して更新しなければならない。
