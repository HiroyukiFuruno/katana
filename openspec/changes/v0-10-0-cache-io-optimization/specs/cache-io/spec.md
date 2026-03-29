## ADDED Requirements

### Requirement: ディレクトリ単位のKVSキャッシュストレージ

システムは、単一のJSONファイル（`cache.json`）の代わりに、専用ディレクトリ（例: `kv/`）内の複数ファイルを用いたKVS（Key-Value Store）形式で永続化キャッシュデータを保存しなければならない（SHALL）。

#### Scenario: キャッシュ値の個別ファイルへの書き込み

- **WHEN** システムが特定のキー（例: `workspace_tabs:/path/to/ws`）に対するキャッシュデータを保存する時
- **THEN** キー名称をSHA-256などでハッシュ化した安全な文字列をファイル名（例: `kv/<namespace>/<hash>.json`）とし、そのキーのデータのみが独立してディスクへ同期される
- **THEN** ファイル内容には `storage_version` と canonical key metadata が含まれ、filename hash だけに依存せず元の論理キーを検証できる
- **THEN** 他の persistent key の値を再シリアライズして同時に書き戻さない

#### Scenario: キャッシュ値のディスクからの遅延読み込み (Lazy Load)

- **WHEN** システムが特定のキーに対するキャッシュデータを要求した際、オンメモリのキャッシュマップに該当データが存在しない時
- **THEN** 該当するハッシュ名のファイルが存在すればディスクから読み込んでオンメモリに保持し、存在しなければ `None` を返す
- **THEN** lookup のために `kv` ディレクトリ全体を走査しない

### Requirement: Persistent entry files are self-describing

システムは、各 persistent entry file に filename hash だけでは失われる論理キー情報と format version を保持しなければならない（SHALL）。

#### Scenario: workspace_tabs entry envelope

- **WHEN** システムが `workspace_tabs` namespace の entry を保存する時
- **THEN** file content には少なくとも `storage_version`, `key.namespace`, `key.workspace_path`, `value` が含まれる
- **THEN** restart 後の read path では `workspace_path` から logical key を再計算して同じ file を参照できる

#### Scenario: diagram entry envelope

- **WHEN** システムが `diagram` namespace の entry を保存する時
- **THEN** file content には少なくとも `storage_version`, `key.namespace`, `key.document_path`, `key.diagram_kind`, `key.source_hash`, `key.theme`, `value` が含まれる
- **THEN** 障害解析や migration 時に file content から元の diagram 対象を検証できる

### Requirement: Persistent cache keys are namespace-aware

システムは、persistent cache key を namespace-aware に扱い、内部ロジックが `workspace_tabs` や `diagram` などの用途境界を明示的に判別できなければならない（SHALL）。

#### Scenario: ワークスペース状態を保存する

- **WHEN** システムがワークスペースの open tabs 状態を保存する時
- **THEN** そのキーは `workspace_tabs` namespace として識別可能である
- **THEN** filename codec は namespace 情報を失わずに安全な永続化表現へ変換できる
- **THEN** restart 後は workspace path から同じ logical key と filename を再計算して復元できる

#### Scenario: 図解キャッシュを保存する

- **WHEN** システムが diagram render 結果を保存する時
- **THEN** そのキーは `diagram` namespace として識別可能である
- **THEN** workspace state と diagram cache の invalidate / clear / migration を独立して扱える
- **THEN** 必要に応じて file content の metadata から対象 document path や diagram kind を検証できる

### Requirement: Callers pass canonical raw keys to CacheFacade

システムは、persistent cache caller が opaque storage key ではなく canonical raw key を `CacheFacade` に渡すようにしなければならない（SHALL）。

#### Scenario: workspace caller uses canonical raw key

- **WHEN** ワークスペース状態を保存・復元する caller が persistent cache を利用する時
- **THEN** caller は `workspace_tabs:{workspace_path}` に相当する canonical raw key を helper 経由で生成する
- **THEN** storage 層がその raw key から namespace と filename を決定する

#### Scenario: diagram caller stops using legacy opaque key

- **WHEN** 図解キャッシュを保存・復元する caller が persistent cache を利用する時
- **THEN** caller は `diagram` namespace の canonical raw key を helper 経由で生成する
- **THEN** caller は旧来の `diagram_<hash>` を新規 write path では使用しない

### Requirement: 旧形式（単一ファイル）からの互換性マイグレーション

システムは新アーキテクチャの導入後、初回の起動時に従来の単一キャッシュファイルから新KVSアーキテクチャへのデータの受け渡しを行わなければならない（SHALL）。

#### Scenario: 旧 cache.json が存在する場合の起動処理

- **WHEN** `DefaultCacheService` の初期化時に、新KVSアーキテクチャでありながら旧形式の `cache.json` がディスク上に存在する時
- **THEN** `cache.json`内のエントリは namespace ごとの方針に従ってKVSディレクトリへ移送される
- **THEN** `workspace_tabs` などの critical namespace が新 store で読めることを確認するまでは、旧 `cache.json` は安全のため保持される
- **THEN** critical namespace の移送と基本検証が完了した後に限り、旧 `cache.json` は削除または `.bak` 等へ退避される
- **THEN** `workspace_tabs` などのユーザー作業状態は upgrade で失われない
- **THEN** 旧 `diagram_<hash>` entry は metadata を復元できないため、新形式 migration の保証対象には含めず、再生成可能データとしてスキップできる

#### Scenario: migration 完了後の runtime 動作

- **WHEN** 旧 `cache.json` からの migration が正常に完了した後にアプリが動作する時
- **THEN** persistent cache の正規 runtime 経路は KVS ディレクトリのみを利用する
- **THEN** `cache.json` を通常の read/write backend として併用しない

### Requirement: Migration is idempotent and crash-safe

システムは、旧 `cache.json` から新 store への migration を再実行可能かつクラッシュ耐性のある形で行わなければならない（SHALL）。

#### Scenario: partial migration is retried safely

- **WHEN** migration の途中でアプリが終了し、`kv` に一部の file だけが存在する状態で再起動した時
- **THEN** migration は再実行でき、同じ canonical key から同じ target filename へ収束する
- **THEN** 部分生成済み file があっても `workspace_tabs` の移送結果を壊さない

#### Scenario: critical namespace migration failure

- **WHEN** `workspace_tabs` などの critical namespace の移送に失敗した時
- **THEN** migration は未完了として扱われ、旧 `cache.json` は再試行のため保持される
- **THEN** 旧 `cache.json` を削除して成功扱いにしない

#### Scenario: best-effort namespace migration failure

- **WHEN** `diagram` namespace の移送に失敗した時
- **THEN** `workspace_tabs` が保全されていれば migration は継続可能である
- **THEN** 失敗した `diagram` entry は破棄または無視され、後続の再生成に委ねられる

#### Scenario: legacy diagram entries are not upgraded losslessly

- **WHEN** 旧 `cache.json` に `diagram_<hash>` のような opaque diagram key が保存されている時
- **THEN** 新 storage はその key から `document_path` や `diagram_kind` を復元しようとしない
- **THEN** その entry は best-effort cache として migration 対象外にできる

### Requirement: 互換性保証の境界を明示する

システムは、永続化フォーマット変更に伴う互換性保証の範囲を明確に定義しなければならない（SHALL）。

#### Scenario: upgrade compatibility

- **WHEN** ユーザーが旧版から新 storage へアップグレードする時
- **THEN** 少なくとも `workspace_tabs` などのユーザーコンテキスト保持に必要な namespace は互換 migration の保証対象である
- **THEN** 新 storage entry には将来 migration のための `storage_version` が保存される

#### Scenario: downgrade compatibility is not guaranteed

- **WHEN** ユーザーが新 storage を生成した後に旧版アプリへ戻す時
- **THEN** 旧版アプリが新 storage を読めることは保証しない
- **THEN** その制約は proposal / design / release note のいずれかで明示される
