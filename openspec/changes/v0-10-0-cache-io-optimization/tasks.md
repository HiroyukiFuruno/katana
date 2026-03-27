## 1. Core Platform (Cache I/O)

- [ ] 1.1 `cache.rs` の `DefaultCacheService` を改修し、単一の `PersistentData` ではなくキーごとの Lazy Load を支えるオンメモリ構造（`RwLock<HashMap<String, String>>` 等）へ変更する
- [ ] 1.2 `get_persistent` において、指定されたキーのSHA-256ハッシュ値をファイル名として、`kv/<hash>.json` からデータを読み込む（Lazy Load）ロジックを実装する
- [ ] 1.3 `set_persistent` において、指定キーのデータを `kv/<hash>.json` として個別ファイルに保存し、非同期または即座にディスクへ同期する処理を実装する
- [ ] 1.4 初期化（`new`）時、旧 `cache.json` が存在する場合はその中身をパースして全キーを `kv` ディレクトリへ移行（分割保存）し、元ファイルをリネーム退避するマイグレーション処理を実装する

## 2. Testing & Verification

- [ ] 2.1 `cache.rs` 内の単体テスト（`test_in_memory_cache_service`等）をKVS分割形式に合わせて修正し、Coverage 100% を維持・拡充する
- [ ] 2.2 既存のUI操作（タブ切替、起動時の状態復元、画像キャッシュ復元等）による連携テストを実行し、デグレがないかを検証する
- [ ] 2.3 キャッシュ全体クリア（`clear_all_directories_in`）が新しい `kv` ディレクトリ構造でも正常に作動し、孤立したファイルが一切残らないことを確認する
