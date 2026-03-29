## MODIFIED Requirements

### Requirement: GitHub Flavored Markdown is supported in preview output

The system SHALL parse and render GitHub Flavored Markdown constructs supported by the chosen Markdown engine, including nested task lists, without introducing duplicate list markers for task items.

#### Scenario: Render common GFM structures

- **WHEN** the active document contains headings, lists, fenced code blocks, and tables supported by the Markdown engine
- **THEN** the preview output preserves those structures in rendered form
- **THEN** unsupported content degrades gracefully without crashing the application

#### Scenario: Render a task list item that owns nested children

- **WHEN** the active document contains a task list item whose line begins with a checkbox marker and whose children contain nested bullet or ordered list items
- **THEN** the parent line is rendered with the checkbox as its only leading marker
- **THEN** the nested child list retains the existing bullet / ordered marker style and indentation rules

## ADDED Requirements

### Requirement: Active Markdown documents use hash-managed disk refresh without implicit save

The system SHALL track a content hash for the last disk state imported into the active Markdown document, and SHALL use that hash for both user-triggered refresh and periodic automatic refresh without performing an implicit save.

#### Scenario: Initialize or advance the imported disk hash after successful synchronization

- **WHEN** the active Markdown document is loaded from disk, saved successfully, or reloaded successfully
- **THEN** the stored last imported disk hash is updated to match the synchronized disk contents
- **THEN** subsequent refresh decisions compare against that updated hash

#### Scenario: Manual refresh with unchanged hash

- **WHEN** the user triggers the shared refresh action and the current on-disk content hash matches the last imported disk hash
- **THEN** the system skips document reload
- **THEN** the in-memory buffer remains unchanged

#### Scenario: Reload a clean document after an external edit

- **WHEN** the active Markdown document has no unsaved changes and its source file hash differs from the last imported disk hash
- **THEN** the system reloads the file contents into the in-memory document buffer
- **THEN** the stored disk hash is updated to the new imported value
- **THEN** the preview is rerendered from the reloaded buffer, including supported diagram blocks
- **THEN** the document remains marked as clean

#### Scenario: Automatic refresh detects an external edit on a clean document

- **WHEN** automatic refresh polling is enabled and the active Markdown document is clean
- **THEN** the system periodically checks whether the current on-disk content hash differs from the last imported disk hash
- **THEN** it reloads and rerenders only when the hash changed

#### Scenario: Refresh a dirty document

- **WHEN** the active Markdown document has unsaved in-memory changes and either manual or automatic refresh detects that the on-disk content hash changed
- **THEN** the system MUST NOT silently replace the in-memory buffer with on-disk contents
- **THEN** the preview is refreshed from the current in-memory buffer instead
- **THEN** the user is shown a recoverable warning that disk reload was skipped because the document is dirty

#### Scenario: Dirty document warning is not repeated for the same external hash

- **WHEN** automatic refresh polling repeatedly observes the same changed on-disk content hash while the active document remains dirty
- **THEN** the system records that an external change is pending for that hash
- **THEN** it MUST NOT repeatedly emit the same warning on every polling interval
- **THEN** the pending state is cleared only after a successful save, a successful reload, or the on-disk hash returns to the stored imported hash

#### Scenario: Reload fails because the source file cannot be read

- **WHEN** the active Markdown document is clean and refresh cannot read the source file from disk
- **THEN** the current in-memory buffer is preserved
- **THEN** the stored last imported disk hash remains unchanged
- **THEN** the user is shown a recoverable error state
