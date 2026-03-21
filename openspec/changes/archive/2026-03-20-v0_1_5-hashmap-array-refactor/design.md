## Context

KatanA Desktop は設定管理、状態保持、シリアライズにおいてRustの `HashMap` や固定長配列 `[T; N]` を活用してきました。しかし、`HashMap` はイテレーション順序が非決定（SipHashによるランダム化）であり、かつジェネリックな Key-Value ストアであるためドメインモデルの型安全性を損なう要因となります。これらを厳格な Data Class の List (`Vec<Struct>`) へ強制移行します。

## Goals / Non-Goals

**Goals:**
- KatanA ワークスペース全体から `HashMap` および固定長・リテラル配列を完全に排除する。
- ドメイン固有の構造体（Data Class）と `Vec` を用いた型安全なリスト表現を強制する。
- 恒久的な再発防止策として `ast_linter.rs` に静的解析ルール（HashMap禁止・配列禁止）を追加する。
- `settings.json` などの既存設定構造の変化に対する自動マイグレーションパスを提供する。

**Non-Goals:**
- Rustのエコシステムやサードパーティクレート内部での `HashMap`/配列 利用を制限すること。
- O(1) パフォーマンスが絶対的にREQUIREとされる箇所（キャッシュルックアップ等）への過剰な最適化（今回は100要素以下の設定・辞書データが大半であるためO(N)で問題なしと判断）。

## Decisions

- **`BTreeMap` ではなく `Vec<DomainStruct>` を採用**:
  単純な順序付きMap (`BTreeMap`) に逃げるのではなく、ユーザー要望の「Data Classへの変換とListの利用」に忠実にするため、キー・バリューのペアを明確な構造体（例: `ExtraSetting { key: String, value: String }`）に定義し、その `Vec` として保持します。
- **Linter の AST トラバーサル**:
  `syn` クレートのVisitorパターンを拡張し、型のパス (`PathSegment`) に `HashMap` が含まれるケース、および `syn::Type::Array` や `syn::Expr::Array` を使用しているケースをトラップし、`MD000` 相当のカスタムエラーを出力します。

## Risks / Trade-offs

- **[Risk] 設定ファイルの非互換クラッシュ**: `settings.json` の `extra` 等がオブジェクト `{}` から配列 `[]` に変わるため、v0.1.3以前のユーザーがv0.1.4を起動するとデシリアライズエラーになります。
  - **Mitigation**: Serdeが型解決を行う前の `serde_json::Value` 段階で走る `MigrationStrategy` に、旧バージョンのオブジェクトを配列にコンバートするロジック（`Migration0_1_3_to_0_1_4` 等）を組み込みます。
- **[Risk] 検索パフォーマンスの低下**: `HashMap` の O(1) 検索に対し、`Vec` の線形探索は O(N) となります。
  - **Mitigation**: 影響を受けるデータ（言語定義、テーマ設定、開いているタブ情報等）の N は極めて小さく、メモリの局所性が高まる `Vec` の方が実質的なパフォーマンスオーバーヘッドは計測不能レベルに収まります。
