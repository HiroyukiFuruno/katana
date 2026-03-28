## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: Supported diagram block payloads are explicitly constrained
The system SHALL accept only the following diagram payload formats in the MVP preview pipeline: raw Mermaid source in fenced `mermaid` blocks, raw PlantUML source including `@startuml` and `@enduml` in fenced `plantuml` blocks, and raw uncompressed Draw.io XML containing `<mxfile>` or `<mxGraphModel>` in fenced `drawio` blocks.

#### Scenario: Accept a supported Mermaid payload
- **WHEN** the active Markdown document contains a fenced `mermaid` block with Mermaid source text
- **THEN** the block is treated as a supported diagram payload

#### Scenario: Accept a supported PlantUML payload
- **WHEN** the active Markdown document contains a fenced `plantuml` block with explicit `@startuml` and `@enduml` delimiters
- **THEN** the block is treated as a supported diagram payload

#### Scenario: Accept a supported Draw.io payload
- **WHEN** the active Markdown document contains a fenced `drawio` block with raw uncompressed XML containing `<mxfile>` or `<mxGraphModel>`
- **THEN** the block is treated as a supported diagram payload

#### Scenario: Reject unsupported diagram encodings
- **WHEN** a diagram block relies on compressed XML, base64 payloads, or external file references that are outside the MVP input contract
- **THEN** the block is handled as an unsupported payload and rendered through the diagram failure fallback path

### Requirement: Mermaid blocks render inline in the standard preview
The system SHALL render fenced Markdown blocks labeled `mermaid` as inline diagrams in the default preview experience.

#### Scenario: Render a Mermaid flowchart
- **WHEN** the active Markdown document contains a valid fenced `mermaid` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** changes to the block are reflected when the preview refreshes

### Requirement: PlantUML blocks render inline in the standard preview
The system SHALL render fenced Markdown blocks labeled `plantuml` as inline diagrams in the default preview experience.

#### Scenario: Render a PlantUML sequence diagram
- **WHEN** the active Markdown document contains a valid fenced `plantuml` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** the rendered result is produced through a fully local bundled rendering path compatible with the desktop application

### Requirement: Draw.io blocks render inline in the standard preview
The system SHALL render fenced Markdown blocks labeled `drawio` as inline diagrams in the default preview experience.

#### Scenario: Render an embedded Draw.io diagram block
- **WHEN** the active Markdown document contains a valid fenced `drawio` block
- **THEN** the preview shows the rendered diagram instead of the raw fenced source
- **THEN** the diagram is rendered without requiring the user to install a separate viewer

### Requirement: Diagram rendering failures do not collapse Markdown preview
The system MUST preserve the preview workflow when a supported diagram block cannot be rendered.

#### Scenario: Fail gracefully on an invalid or unsupported diagram payload
- **WHEN** a supported diagram block cannot be rendered successfully
- **THEN** the preview remains available for the rest of the Markdown document
- **THEN** the failing block is replaced with a clear fallback state that exposes the source and error context
