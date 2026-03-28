## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: Markdown documents can be edited as local workspace files
The system SHALL allow the active Markdown document to be edited in memory and saved back to its workspace file.

#### Scenario: Modify the active Markdown document
- **WHEN** the user types into the active Markdown editor
- **THEN** the system updates the in-memory document buffer
- **THEN** the document is marked as having unsaved changes

#### Scenario: Save edits to disk
- **WHEN** the user saves a dirty Markdown document
- **THEN** the system writes the current buffer to the document's file path in the active workspace
- **THEN** the document is marked as clean after a successful write

#### Scenario: Editing does not implicitly save the source file
- **WHEN** the active Markdown buffer changes without an explicit save action
- **THEN** the workspace file contents remain unchanged on disk
- **THEN** the document remains marked as having unsaved changes

### Requirement: Preview rendering stays synchronized with the active buffer
The system SHALL render preview output from the current in-memory Markdown buffer rather than from the last saved file contents.

#### Scenario: Update preview after an edit
- **WHEN** the active Markdown buffer changes
- **THEN** the preview renderer uses the updated buffer contents
- **THEN** the preview pane reflects the edit without requiring the file to be saved first

### Requirement: GitHub Flavored Markdown is supported in preview output
The system SHALL parse and render GitHub Flavored Markdown constructs supported by the chosen Markdown engine.

#### Scenario: Render common GFM structures
- **WHEN** the active document contains headings, lists, fenced code blocks, and tables supported by the Markdown engine
- **THEN** the preview output preserves those structures in rendered form
- **THEN** unsupported content degrades gracefully without crashing the application
