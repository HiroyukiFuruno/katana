# v0.6.2 Tasks

## 1. 数式レンダリング (KaTeX対応)

- [x] 1.1 Rust用KaTeXライブラリ調査・選定（`katex-rs`, `pulldown-cmark-katex` 等）
- [x] 1.2 `egui_commonmark` に数式レンダリング統合
- [x] 1.3 ブロック数式・インライン数式・1行数式の描画対応（フォールバック: LaTeX source をコードブロック表示）
- [ ] 1.4 fixture で目視確認

## 2. テストフィクスチャ基盤整備

- [x] 2.1 `tests/fixtures/` の復旧（symlink or build script copy）
- [x] 2.2 `sample.md` 結合fixture の再生成 or テストパス修正
- [x] 2.3 全27テストの復旧確認

## 3. 描画品質改善

- [x] 3.1 インラインコード背景の上下中央補正
- [x] 3.2 `<u>` 下線の描画品質確認
- [x] 3.3 `<mark>` ハイライトの描画品質確認

## 4. アコーディオン描画

- [x] 4.1 `<details>/<summary>` の折りたたみUI対応
- [ ] 4.2 開閉アニメーション・スタイリング
- [ ] 4.3 fixture で目視確認
