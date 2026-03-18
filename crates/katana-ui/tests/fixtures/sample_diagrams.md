# 🧪 KatanA Rendering — Diagrams (External Dependencies)

This fixture exercises diagram rendering that depends on external tools:
Mermaid (mmdc), PlantUML (jar), and DrawIo (Pure Rust).

<p align="center">
  English | <a href="sample_diagrams.ja.md">日本語</a>
</p>

---

## 7. Diagrams — Mermaid

### 7.1 Flowchart

```mermaid
graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Process A]
    B -->|No| D[Process B]
    C --> E[End]
    D --> E
```

### 7.2 Sequence Diagram

```mermaid
sequenceDiagram
    participant User
    participant KatanA
    participant FileSystem

    User->>KatanA: Open file
    KatanA->>FileSystem: Read
    FileSystem-->>KatanA: Markdown text
    KatanA-->>User: Render preview
```

### 7.3 Class Diagram

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

### 7.4 State Diagram

```mermaid
stateDiagram-v2
    [*] --> Pending
    Pending --> Image : render success
    Pending --> Error : render failure
    Pending --> CommandNotFound : tool missing
    Pending --> NotInstalled : jar missing
    Image --> [*]
    Error --> [*]
    CommandNotFound --> [*]
    NotInstalled --> [*]
```

### 7.5 Gantt Chart

```mermaid
gantt
    title KatanA Development Schedule
    dateFormat  YYYY-MM-DD
    section Core
    Markdown Rendering    :done, 2026-01-01, 30d
    Diagram Support       :done, 2026-02-01, 28d
    section UI
    Preview Pane          :done, 2026-01-15, 45d
    Theme Support         :active, 2026-03-01, 30d
    section Testing
    Unit Tests            :done, 2026-02-01, 28d
    Integration Tests     :active, 2026-03-01, 30d
```

---

## 8. Diagrams — PlantUML

### 8.1 Sequence Diagram

```plantuml
@startuml
actor User
participant "KatanA" as K
database "FileSystem" as FS

User -> K: Open file
K -> FS: Read markdown
FS --> K: Content
K --> User: Render preview
@enduml
```

### 8.2 Class Diagram

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

### 8.3 Activity Diagram

```plantuml
@startuml
start
:Load Markdown;
if (Diagram block?) then (yes)
    :Render in background thread;
    if (Tool installed?) then (yes)
        :Generate Image section;
    else (no)
        :NotInstalled / CommandNotFound;
    endif
else (no)
    :Generate Markdown section;
endif
:Display in UI;
stop
@enduml
```

---

## 9. Diagrams — DrawIo

### 9.1 Basic Shapes

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

### 9.2 Multiple Shapes with Connections

```drawio
<mxGraphModel>
  <root>
    <mxCell id="0"/>
    <mxCell id="1" parent="0"/>
    <mxCell id="2" value="Input" style="shape=parallelogram;fillColor=#fff2cc;strokeColor=#d6b656;" vertex="1" parent="1">
      <mxGeometry x="50" y="30" width="120" height="50" as="geometry"/>
    </mxCell>
    <mxCell id="3" value="Process" style="rounded=1;fillColor=#dae8fc;strokeColor=#6c8ebf;" vertex="1" parent="1">
      <mxGeometry x="50" y="120" width="120" height="50" as="geometry"/>
    </mxCell>
    <mxCell id="4" value="Output" style="shape=parallelogram;fillColor=#d5e8d4;strokeColor=#82b366;" vertex="1" parent="1">
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

## 10. Mixed Content with Diagrams

KatanA rendering pipeline:

```mermaid
graph LR
    MD[Markdown Source] --> Parser
    Parser --> Sections[RenderedSections]
    Sections --> UI[egui Preview]
```

Proper spacing between the flowchart above and this text.

And a DrawIo diagram below:

```drawio
<mxGraphModel>
  <root>
    <mxCell id="0"/>
    <mxCell id="1" parent="0"/>
    <mxCell id="2" value="Mixed Content Test" style="rounded=1;fillColor=#f8cecc;strokeColor=#b85450;" vertex="1" parent="1">
      <mxGeometry x="50" y="30" width="200" height="60" as="geometry"/>
    </mxCell>
  </root>
</mxGraphModel>
```

↑ All sections should render correctly without overlapping.

---

## 12. Consecutive Diagrams

Three diagram types in a row. One failing should not affect the others.

```mermaid
pie title Rendering Engine Distribution
    "DrawIo (Rust)" : 1
    "Mermaid (mmdc)" : 1
    "PlantUML (jar)" : 1
```

```drawio
<mxGraphModel>
  <root>
    <mxCell id="0"/>
    <mxCell id="1" parent="0"/>
    <mxCell id="2" value="Between Diagrams" style="rounded=1;" vertex="1" parent="1">
      <mxGeometry x="50" y="30" width="150" height="50" as="geometry"/>
    </mxCell>
  </root>
</mxGraphModel>
```

```plantuml
@startuml
Alice -> Bob : OK
Bob --> Alice : Done
@enduml
```

↑ All three diagrams rendered independently with proper spacing.

---

## ✅ Verification Complete

If all sections above render correctly, diagram rendering is working.
