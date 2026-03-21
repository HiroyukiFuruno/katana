## 1. Linter ルール実装

- [x] 1.1 `ast_linter.rs` に `HashMap` 検出と禁止ルールを追加する
- [x] 1.2 `ast_linter.rs` に固定長配列 `[T; N]` およびリテラル配列の禁止ルールを追加する

## 2. コア設定とマイグレーション

- [x] 2.1 `settings.rs` の `extra` を `Vec<ExtraSetting>` にリファクタリングする
- [x] 2.2 旧バージョン (v0.1.3以前) の JSON 互換用マイグレーション処理 (`Migration0_1_3_to_0_1_4`) を追加する

## 3. ドメインモデルのリスト化 (State & AI)

- [x] 3.1 `ai/mod.rs` の `params`, `metadata`, `providers` を `Vec<Struct>` 形式に変換する
- [x] 3.2 `app_state.rs` のタブ状態管理 (`tab_view_modes`, `tab_split_states`) を `Vec` に変換する

## 4. UI状態とキャッシュモデルのリスト化

- [x] 4.1 `i18n.rs` の言語辞書内部構造を `HashMap` から `Vec<I18nDictionaryEntry>` に変換する
- [x] 4.2 `svg_loader.rs` および `http_cache_loader.rs` の内部キャッシュを `Vec` に変換する
- [x] 4.3 `shell.rs`, `plugin/mod.rs`, `emoji.rs`, `drawio_renderer.rs` のコンテキストを `Vec` に置き換える

## 5. プリミティブ配列の廃止 (Array to Vec)

- [x] 5.1 Linter ルール定義などで使われている `&[&str]` や固定長配列定義を `Vec<T>` に置き換える
- [x] 5.2 コードベース全体の配列リテラル `[a, b]` を `vec![a, b]` に置換する

## 6. クリーンアップとテスト

- [x] 6.1 `Vec` 化によってコンパイルエラーとなる箇所の呼び出し側やテストコードを修正する
- [x] 6.2 `cargo test --all` を実行し、全件通過することと Linter がクリーンであることを確認する
