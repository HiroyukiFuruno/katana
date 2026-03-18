# 🧪 KatanA Rendering — HTML Centering

This fixture exercises HTML centering (past bug: left-aligned instead of centered).

<p align="center">
  English | <a href="sample_html.ja.md">日本語</a>
</p>

---

## 1. HTML Centering (Past Bug: Elements Left-Aligned Instead of Centered)

### 1.1 `<h1 align="center">` — Centered Heading

<h1 align="center">KatanA Desktop</h1>

↑ The heading "KatanA Desktop" should be **horizontally centered** in the panel.

### 1.2 `<p align="center">` — Centered Paragraph

<p align="center">
  A fast, lightweight Markdown workspace for macOS — built with Rust and egui.
</p>

↑ The description text should be **horizontally centered** in the panel.

### 1.3 Multiple Consecutive Centered Blocks

<h1 align="center">Centered Heading</h1>

<p align="center">
  Centered description paragraph.
</p>

<p align="center">
  Second centered paragraph — should NOT overlap with the first one.
</p>

↑ All three elements should be on separate lines, all centered.

### 1.4 Badge Row (Multiple Link Images on Same Line)

<p align="center">
  <a href="#"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="#"><img src="https://img.shields.io/badge/CI-passing-brightgreen.svg" alt="CI"></a>
  <a href="#"><img src="https://img.shields.io/badge/platform-macOS-lightgrey" alt="Platform: macOS"></a>
</p>

↑ Three badges should appear on the **same line**, centered.
(If they are on separate lines, that's a bug.)

### 1.5 Mixed Text + Link Centering

<p align="center">
  English | <a href="#">日本語</a>
</p>

↑ "English | 日本語" should appear on the same line, centered.

### 1.6 Full README Header Reproduction

<p align="center">
  <img src="https://via.placeholder.com/128" width="128" alt="icon">
</p>

<h1 align="center">KatanA Desktop</h1>

<p align="center">
  A fast, lightweight Markdown workspace for macOS
</p>

<p align="center">
  <a href="#"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <a href="#"><img src="https://img.shields.io/badge/CI-passing-brightgreen.svg" alt="CI"></a>
</p>

<p align="center">
  English | <a href="#">日本語</a>
</p>

↑ Icon → heading → description → badges → language selector, all centered in order.

---

## ✅ Verification Complete

If all sections above render correctly, HTML centering is working.
