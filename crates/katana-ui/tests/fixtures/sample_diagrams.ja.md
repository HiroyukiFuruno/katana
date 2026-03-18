# 🧪 KatanA 描画 — ダイアグラム（外部依存）

このフィクスチャは外部ツール依存のダイアグラム描画を検証します：
Mermaid (mmdc)、PlantUML (jar)、DrawIo (純Rust)。

<p align="center">
  <a href="sample_diagrams.md">English</a> | 日本語
</p>

---

## 7. ダイアグラム — Mermaid

### 7.1 フローチャート

```mermaid
graph TD
    A[開始] --> B{判定}
    B -->|Yes| C[処理A]
    B -->|No| D[処理B]
    C --> E[終了]
    D --> E
```

### 7.2 シーケンス図

```mermaid
sequenceDiagram
    participant ユーザー
    participant KatanA
    participant ファイルシステム

    ユーザー->>KatanA: ファイルを開く
    KatanA->>ファイルシステム: 読み取り
    ファイルシステム-->>KatanA: Markdownテキスト
    KatanA-->>ユーザー: プレビュー表示
```

### 7.3 クラス図

```mermaid
classDiagram
    class PreviewPane {
        +Vec~RenderedSection~ sections
        +full_render(source, path)
        +wait_for_renders()
        +show_content(ui)
    }
    class RenderedSection {
        <<enumeration>>
        Markdown
        Image
        Error
        CommandNotFound
        NotInstalled
        Pending
    }
    PreviewPane --> RenderedSection
```

### 7.4 状態遷移図

```mermaid
stateDiagram-v2
    [*] --> Pending
    Pending --> Image : 描画成功
    Pending --> Error : 描画失敗
    Pending --> CommandNotFound : ツール未検出
    Pending --> NotInstalled : jar未インストール
    Image --> [*]
    Error --> [*]
    CommandNotFound --> [*]
    NotInstalled --> [*]
```

### 7.5 ガントチャート

```mermaid
gantt
    title KatanA 開発スケジュール
    dateFormat  YYYY-MM-DD
    section コア
    Markdownレンダリング    :done, 2026-01-01, 30d
    ダイアグラム対応         :done, 2026-02-01, 28d
    section UI
    プレビューペイン        :done, 2026-01-15, 45d
    テーマ対応              :active, 2026-03-01, 30d
    section テスト
    ユニットテスト           :done, 2026-02-01, 28d
    インテグレーションテスト  :active, 2026-03-01, 30d
```

---

## 8. ダイアグラム — PlantUML

### 8.1 シーケンス図

```plantuml
@startuml
actor ユーザー
participant "KatanA" as K
database "ファイルシステム" as FS

ユーザー -> K: ファイルを開く
K -> FS: Markdown読み込み
FS --> K: コンテンツ
K --> ユーザー: プレビュー描画
@enduml
```

### 8.2 クラス図

```plantuml
@startuml
class PreviewPane {
    +sections: Vec<RenderedSection>
    +full_render(source, path)
    +show_content(ui)
}

enum RenderedSection {
    Markdown
    Image
    Error
    Pending
}

PreviewPane --> RenderedSection
@enduml
```

### 8.3 アクティビティ図

```plantuml
@startuml
start
:Markdownを読み込む;
if (ダイアグラムブロック?) then (yes)
    :バックグラウンドスレッドで描画;
    if (ツールがインストール済み?) then (yes)
        :Image セクション生成;
    else (no)
        :NotInstalled / CommandNotFound;
    endif
else (no)
    :Markdown セクション生成;
endif
:UIに表示;
stop
@enduml
```

---

## 9. ダイアグラム — DrawIo

### 9.1 基本図形

```drawio
<mxGraphModel>
  <root>
    <mxCell id="0"/>
    <mxCell id="1" parent="0"/>
    <mxCell id="2" value="Hello" style="rounded=1;fillColor=#dae8fc;strokeColor=#6c8ebf;" vertex="1" parent="1">
      <mxGeometry x="50" y="50" width="120" height="60" as="geometry"/>
    </mxCell>
    <mxCell id="3" value="World" style="ellipse;fillColor=#d5e8d4;strokeColor=#82b366;" vertex="1" parent="1">
      <mxGeometry x="250" y="50" width="120" height="60" as="geometry"/>
    </mxCell>
    <mxCell id="4" style="edgeStyle=orthogonalEdgeStyle;" edge="1" source="2" target="3" parent="1">
      <mxGeometry relative="1" as="geometry"/>
    </mxCell>
  </root>
</mxGraphModel>
```

### 9.2 複数の図形と接続

```drawio
<mxGraphModel>
  <root>
    <mxCell id="0"/>
    <mxCell id="1" parent="0"/>
    <mxCell id="2" value="入力" style="shape=parallelogram;fillColor=#fff2cc;strokeColor=#d6b656;" vertex="1" parent="1">
      <mxGeometry x="50" y="30" width="120" height="50" as="geometry"/>
    </mxCell>
    <mxCell id="3" value="処理" style="rounded=1;fillColor=#dae8fc;strokeColor=#6c8ebf;" vertex="1" parent="1">
      <mxGeometry x="50" y="120" width="120" height="50" as="geometry"/>
    </mxCell>
    <mxCell id="4" value="出力" style="shape=parallelogram;fillColor=#d5e8d4;strokeColor=#82b366;" vertex="1" parent="1">
      <mxGeometry x="50" y="210" width="120" height="50" as="geometry"/>
    </mxCell>
    <mxCell id="5" edge="1" source="2" target="3" parent="1">
      <mxGeometry relative="1" as="geometry"/>
    </mxCell>
    <mxCell id="6" edge="1" source="3" target="4" parent="1">
      <mxGeometry relative="1" as="geometry"/>
    </mxCell>
  </root>
</mxGraphModel>
```

---

## 10. ダイアグラム混在コンテンツ

KatanA の描画パイプライン:

```mermaid
graph LR
    MD[Markdown ソース] --> Parser
    Parser --> Sections[RenderedSections]
    Sections --> UI[egui プレビュー]
```

上のフローチャートとこのテキストの間にスペースがあること。

そして下に DrawIo:

```drawio
<mxGraphModel>
  <root>
    <mxCell id="0"/>
    <mxCell id="1" parent="0"/>
    <mxCell id="2" value="混在コンテンツテスト" style="rounded=1;fillColor=#f8cecc;strokeColor=#b85450;" vertex="1" parent="1">
      <mxGeometry x="50" y="30" width="200" height="60" as="geometry"/>
    </mxCell>
  </root>
</mxGraphModel>
```

↑ すべてのセクションが正しく描画され、互いに重ならないこと。

---

## 12. 複数ダイアグラム連続表示

3種類のダイアグラムを連続で配置。1つの失敗が他に影響しないこと。

```mermaid
pie title 描画エンジン分布
    "DrawIo (Rust)" : 1
    "Mermaid (mmdc)" : 1
    "PlantUML (jar)" : 1
```

```drawio
<mxGraphModel>
  <root>
    <mxCell id="0"/>
    <mxCell id="1" parent="0"/>
    <mxCell id="2" value="ダイアグラム間" style="rounded=1;" vertex="1" parent="1">
      <mxGeometry x="50" y="30" width="150" height="50" as="geometry"/>
    </mxCell>
  </root>
</mxGraphModel>
```

```plantuml
@startuml
Alice -> Bob : OK
Bob --> Alice : 完了
@enduml
```

↑ 3つのダイアグラムがそれぞれ独立して描画され、
テキストなしでも正しくスペーシングされること。

---

## ✅ 検証完了

すべてのセクションが正しく表示されていれば、ダイアグラム描画は正常です。
