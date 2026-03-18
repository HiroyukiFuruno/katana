# 🧪 KatanA 描画 — HTMLセンタリング

このフィクスチャはHTMLセンタリング機能（過去不具合: 中央寄せにならず左寄せになる問題）を検証します。

<p align="center">
  <a href="sample_html.md">English</a> | 日本語
</p>

---

## 1. HTMLセンタリング（過去不具合: 中央寄せにならず左寄せになる問題）

### 1.1 `<h1 align="center">` — 中央寄せ見出し

<h1 align="center">KatanA Desktop</h1>

↑ 見出し「KatanA Desktop」がパネルの **水平中央** に表示されていること。

### 1.2 `<p align="center">` — 中央寄せ段落

<p align="center">
  高速・軽量な macOS 向け Markdown ワークスペース — Rust と egui で構築。
</p>

↑ 説明文がパネルの **水平中央** に表示されていること。

### 1.3 複数のセンタリングブロックが連続する場合

<h1 align="center">中央寄せ見出し</h1>

<p align="center">
  中央寄せの説明段落。
</p>

<p align="center">
  2つ目の中央寄せ段落 — 1つ目と重ならないこと。
</p>

↑ 3つの要素がそれぞれ独立した行に、すべて中央揃えで表示されること。

### 1.4 バッジ行（複数リンク画像の同一行表示）

<p align="center">
  <a href="#"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="#"><img src="https://img.shields.io/badge/CI-passing-brightgreen.svg" alt="CI"></a>
  <a href="#"><img src="https://img.shields.io/badge/platform-macOS-lightgrey" alt="Platform: macOS"></a>
</p>

↑ 3つのバッジが **同一行** に中央揃えで並んでいること。
（個別の行に分かれていたらバグ）

### 1.5 テキスト + リンクの混在センタリング

<p align="center">
  <a href="sample_html.md">English</a> | 日本語
</p>

↑ 「English | 日本語」が中央揃えの同一行に表示されること。

### 1.6 READMEヘッダー完全再現

<p align="center">
  <img src="https://via.placeholder.com/128" width="128" alt="アイコン">
</p>

<h1 align="center">KatanA Desktop</h1>

<p align="center">
  高速・軽量な macOS 向け Markdown ワークスペース
</p>

<p align="center">
  <a href="#"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="#"><img src="https://img.shields.io/badge/CI-passing-brightgreen.svg" alt="CI"></a>
</p>

<p align="center">
  <a href="sample_html.md">English</a> | 日本語
</p>

↑ アイコン→見出し→説明→バッジ群→言語切替 がすべて中央揃えで順番に表示されること。

---

## ✅ 検証完了

すべてのセクションが正しく表示されていれば、HTMLセンタリングは正常です。
