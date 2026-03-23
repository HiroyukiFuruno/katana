## Why

`vendor/egui_commonmark` においてカスタマイズされたコードが直接編集・配置されており、アップストリームのリポジトリとの追随や履歴管理が困難になっているため。正式に `git subtree` を用いた運用へ切り替えたい。

## What Changes

現在の `vendor/egui_commonmark` をリポジトリの履歴から一旦分割/削除し、公式（あるいはフォーク元のリポジトリ）からの `git subtree add` を用いて再構築する。
カスタマイズされた差分は、独自のパッチコミットとしてsubtree化以後に適用し直す。

## Capabilities

### New Capabilities

### Modified Capabilities

- `vendor`: `egui_commonmark` の依存管理方式をコピー＆ペーストから `git subtree` ベースへとモダナイズ

## Impact

依存ライブラリの管理方式が変わるのみであり、プロダクトコードの提供機能自体に差分は生じない。
