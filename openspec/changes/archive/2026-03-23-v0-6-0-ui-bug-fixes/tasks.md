## Branch Rule

Tasks Grouped by ## = Adhere unconditionally to the branching standard defined in the `/openspec-branching` workflow (`.agents/workflows/openspec-branching.md`) throughout your implementation sessions.

## 1. P0 CPU無限ループ および タブ切り替えバグ修正

- [x] 1.1 `KatanaSvgLoader::load` が失敗した場合にエラーを返し、キャッシュさせるようにする (`LoadError::NotSupported` の利用)
- [x] 1.2 `shell.rs` において、`tab.hash` （キャッシュ）を判定し、同一ハッシュなら早期リターンするよう最適化 (CPU100%バグ修正)
- [x] 1.3 スプラッシュ画面の無限再描画ループの修正 (`request_repaint_after` の利用)
- [x] 1.4 エディタのレンダリングループ内に埋め込まれていた大量の `println!` (ゴミ) の削除

### Definition of Done (DoD)

- [x] CPUが100%に張り付く現象が消失し、UIスレッドがアイドル状態に戻ることを確認
- [x] `tests::svg_loader` などが通過する
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

## 2. P1/P3/P4 UI レンダリングバグ修正

- [x] 2.1 `needs_splash` フラグを使い、UI初期化ではなく「更新（初回フレーム）」タイミングでSplashタイマーを開始させる
- [x] 2.2 Blockquote 上下の不要な表示改行を削除する
- [x] 2.3 `KatanaApp` から不要にエラーが出る `bytes://icon/copy.svg` などを事前登録し、アイコンエラーを解消する (内部描画への移行で解決)

### Definition of Done (DoD)

- [x] 起動時に正しくスプラッシュが表示され、テスト `make test` にも退行が生じていない
- [x] Execute `/openspec-delivery` workflow (`.agents/workflows/openspec-delivery.md`) to run the comprehensive delivery routine (Self-review, Commit, PR Creation, and Merge).

---

## 3. Final Verification & Release Work

- [x] 3.1 Execute self-review using `docs/coding-rules.ja.md` and `.agents/skills/self-review/SKILL.md` (Check for missing version updates in each file)
- [x] 3.2 Ensure `make check` passes with exit code 0
- [x] 3.3 Merge the intermediate base branch (derived originally from master) into the `master` branch
- [x] 3.4 Create a PR targeting `master`
- [x] 3.5 Merge into master (※ `--admin` is permitted)
- [x] 3.6 Execute release tagging and creation using `.agents/skills/release_workflow/SKILL.md` for `0.6.0`
- [x] 3.7 Archive this change by leveraging OpenSpec skills like `/opsx-archive`

---

## 4. P0 ゾンビスレッドと外部プロセスの無制限実行バグ修正 (追加対応)

- [x] 4.1 キャンセルトークンの導入
  - `PreviewPane` に `cancel_token: Arc<AtomicBool>` を持たせ、新しく `full_render` する前やドロップ時に以前のトークンを `true` に更新する。
- [x] 4.2 バックグラウンドスレッドの早期終了
  - スレッドプール (`std::thread::spawn`) のループ内でジョブ取り出しごとに `cancel_token` を評価し、`true` なら直ちに `break` でスレッドを抜ける実装を行う。
- [x] 4.3 `shell.rs` でのタブ削除時キャンセル
  - タブを削除するアクション (`handle_close_tab` 等) において、対象の `PreviewPane` の `cancel_token` を `true` にして確実に不要なプロセスを殺す。

### Definition of Done (DoD)

- [x] 巨大なダイアグラムファイル (`sample_diagrams.ja.md`) を開いてからすぐに別タブに移動したりタブを閉じたりした場合、裏で処理が走らず CPU が 0% 近くのアイドル状態にすぐ戻ることを Activity Monitor 等で確認できる。
- [x] TDDで、キャンセルトークンが機能してスレッドが中断されることを証明する（GREEN）。

## 5. UI レンダリングバグ修正 (最終フェーズ)

- [x] 5.1 引用 (Blockquote) 内のリスト表示と余白の修正
  - [x] 引用内のリスト項目が改行されないよう `horizontal_wrapped` 内で描画
  - [x] 引用内のコードブロック上下の余計な空行を削除
- [x] 5.2 コードブロックのコピーボタン表示バグ修正
  - [x] SVG読み込みから `Painter` API による直接描画へ移行 (描画の確実性向上)
  - [x] ボタン位置を 2-3px 下、および右端に寄せる調整 (見栄えとスクロールバー被り防止)
- [x] 5.3 設定画面のレイアウト崩れバグ修正
  - [x] ウィンドウを開いた際に中央パネルが消失・縮小する問題を、サイドパネルの幅固定 (`resizable(false)`) により解消
- [x] 5.4 テストと品質管理
  - [x] `tree_layout` テストでのスプラッシュ画面によるタイムアウト問題を `skip_splash()` により解消
  - [x] `svg_loader` のフォールバックロジックに対するテスト追加とカバレッジゲート通過
  - [x] `make check` (fmt, clippy, test, coverage) の All Pass 確認
