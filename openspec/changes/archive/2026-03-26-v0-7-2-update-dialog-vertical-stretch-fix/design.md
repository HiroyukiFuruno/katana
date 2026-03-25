## Context

`render_update_window` 関数は `egui::Window` に `.resizable(false)` を設定しているが、
内包する `ScrollArea` が `.auto_shrink([false; 2])` のまま残っており、
「最新版です」などコンテンツ量が少い状態でも ScrollArea が縦方向を縮小せず
ウィンドウが不必要に縦伸びする。

## Goals / Non-Goals

**Goals:**
- `ScrollArea::auto_shrink` を `[true, true]` に変更してコンテンツに追従した高さを確保する
- integration test で縦伸び状態（高さが過剰に大きい）を TDD(RED) で再現し、GREEN で修正を担保する

**Non-Goals:**
- ウィンドウの width や他のダイアログへの変更

## Decisions

- `auto_shrink([false; 2])` → `auto_shrink([true, true])` の 1 行変更で対応する
- ScrollArea は update_available 分岐のみに存在するため「最新版」表示では ScrollArea 自体不要だが、  
  既存構成を最小変更で維持する観点から auto_shrink の変更のみに留める
- テストは `katana-ui` integration test スイートに追加し、`render_update_window` が  
  up-to-date 状態で期待値以下の高さに収まることをアサートする

## Risks / Trade-offs

- `auto_shrink([true, true])` に変更した場合、release notes が長い更新がある時には  
  ScrollArea が自動縮小される可能性があるが、`max_height(250.0)` が上限として機能するため問題なし
