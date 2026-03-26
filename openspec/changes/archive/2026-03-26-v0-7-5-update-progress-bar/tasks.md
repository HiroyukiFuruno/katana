# v0.7.5: アップデーターのプログレスバー実装 (Progress Bar Implementation)

## 概要 (Overview)

v0.7.0 / v0.7.1 で実装されたアプリ内アップデーターにおいて、更新処理中（ダウンロード・インストール段階）の進行状態が Spinner のみで表示され、プログレスバーによる視覚的な進捗（DLパーセンテージやファイル展開数）がユーザーに提示されていなかった問題を修正する。

## 目標 (Goals)

1. **ダウンロードの進捗表示**: ZIPファイルの総容量とダウンロード済みバイト数を取得し、プログレスバーを描画する。
2. **インストールの進捗表示**: ZIP展開（Extract）時の全ファイル数と処理済みファイル数を取得し、プログレスバーを描画する。
3. **Modal UI の連携**: `katana-core` 側の非同期処理から `katana-ui` の `mpsc::channel` へ進捗状況をリアルタイム送信し、`egui::ProgressBar` を用いてスムーズに表示する。

---

## 1. コアロジックの非同期・進捗レポート対応 (katana-core)

- [x] 1.1 `UpdateProgress` 列挙体の追加 (`katana_core::update`)
  - `Downloading { downloaded: u64, total: Option<u64> }`
  - `Extracting { current: usize, total: usize }`
- [x] 1.2 `download_update` のシグネチャ変更
  - コールバック関数 `F: FnMut(u64, Option<u64>)` を受け取るようにし、`std::io::copy` の代わりにカスタム `Read` 構造体等を用いてダウンロード進捗を逐次通知するよう改修する。
- [x] 1.3 `extract_update` のシグネチャ変更
  - コールバック関数 `F: FnMut(usize, usize)` を受け取るようにし、`archive.extract` の一括展開ではなく、マニュアルでのエントリごとの展開ループ（+パーミッション復元）へ書き換え、1ファイルごとの進捗を通知する。

## 2. UIフロントエンドとの非同期チャネル連携 (katana-ui)

- [x] 2.1 `shell.rs` の `UpdateInstallProgress` チャネル型の拡張
  - 従来 `Result<UpdatePreparation, String>` 送信専用だったチャネルを、`Event(UpdateProgress)` または `Finished(Result)` を送れるカスタム列挙体 `UpdateInstallEvent` へ変更する。
- [x] 2.2 `shell.rs` の `AppAction::InstallUpdate` トランジション変更
  - スレッド起動時に `prepare_update` へ上記のイベント送信クロージャを渡し、進捗が発生するたびに UI スレッドへ `tx.send()` するクロージャを構築する。

## 3. UI 描画の実装 (katana-ui)

- [x] 3.1 `UpdatePhase` 列挙体への進況状態の追加
  - `Downloading { progress: f32 }`, `Installing { progress: f32 }` のように浮動小数点（0.0〜1.0）を持たせる。
- [x] 3.2 `shell_logic.rs` における非同期受信（ポーリング）時の状態更新
  - チャネルから `UpdateInstallEvent` を受信した際に、`state.update_phase` の `progress` 値を最新のものに更新し、UI再描画 (`ctx.request_repaint()`) をトリガーする。
- [x] 3.3 `shell_ui.rs` におけるプログレスバー描画 (`egui::ProgressBar`)
  - 既存の Spinner 表示領域を、`egui::ProgressBar::new(progress).animate(true)` とテキスト（「ダウンロード中... 45%」など）へアップグレードする。

## 4. 提出とリリース (Verification & Delivery)

- [x] 4.1 テストコード (TDD) のコンパイルおよび実行 (`make check`)
- [ ] 4.2 ローカルサーバーを用いた疑似ダウンロードによるプログレスバー動作の目視確認
- [ ] 4.3 コミット・PR作成を通じた `/openspec-delivery` フローの実行
