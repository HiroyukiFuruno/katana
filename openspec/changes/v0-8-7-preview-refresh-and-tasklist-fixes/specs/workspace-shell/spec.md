## ADDED Requirements

### Requirement: Shared document refresh is available from the common shell chrome

The system SHALL expose a single active-document refresh control from common shell chrome, and that control SHALL remain available regardless of whether the active document is shown in CodeOnly, PreviewOnly, or Split mode.

#### Scenario: Use shared refresh in CodeOnly mode

- **WHEN** an active Markdown document is open and the user switches to CodeOnly mode
- **THEN** the shared refresh control remains visible in the common shell chrome
- **THEN** invoking it applies the same refresh semantics as in other view modes

#### Scenario: Use shared refresh in PreviewOnly or Split mode

- **WHEN** an active Markdown document is open and the user is in PreviewOnly or Split mode
- **THEN** the same shared refresh control is available without requiring a preview-local alternative
- **THEN** there is no second refresh control whose behavior diverges from the shared refresh semantics

#### Scenario: No active document is selected

- **WHEN** there is no active document
- **THEN** the shared refresh control is disabled
- **THEN** invoking refresh does not mutate workspace or preview state
