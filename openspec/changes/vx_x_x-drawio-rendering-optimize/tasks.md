# Tasks: Draw.io レンダリングの最適化と完全対応

- [ ] 1. **Draw.io CLI レンダラの追加** `crates/katana-core/src/markdown/drawio_renderer.rs`
  - 依存としての Draw.io Desktop アプリケーションの存在確認ロジック (`/Applications/draw.io.app/Contents/MacOS/draw.io`) を実装する。
  - `mmdc` と同様に、サブプロセスでバックグラウンド実行してPNG/SVG形式へエクスポートする。
  - カスタムパース実装 (MVP) を削除し、すべて CLI を通すようにする。
- [ ] 2. **NotInstalled 時の UI フォールバック** `crates/katana-ui/src/preview_pane_ui.rs`
  - アプリ未インストール時にダウンロードリンクへ誘導する画面を実装する（Mermaid同様に `RenderedSection::NotInstalled` を返す）。
- [ ] 3. **Markdown の拡張画像マウント対応 (`![alt](file.drawio)`)** `crates/katana-core/src/preview.rs` / `katana-core/src/markdown/mod.rs`
  - egui_commonmark (もしくはカスタムされたレンダラーフック) で `.drawio` を拡張子に持つ画像リンクを検知する。
  - HTTPリクエストによるダウンロードではなく、ローカルファイルとして読み込み、`drawio_renderer` に回す処理を追加。
- [ ] 4. **レンダリング結果のキャッシュ処理の導入**
  - drawioやPlantUMLのCLI実行は重いため、入力ファイルのハッシュ等で重複変換を防ぐためのインメモリキャッシュ（可能であればディスクキャッシュ）をプレビュー基盤に組み込む。
- [ ] 5. **テスト追加・修正** `crates/katana-core/tests/markdown_drawio.rs`
  - `aws.drawio` に相当するテストリソースファイルを `crates/katana-ui/tests/fixtures` 等に配置。
  - CLIを経由した際のレンダリングパターンのチェックを追加。
