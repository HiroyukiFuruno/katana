## MODIFIED Requirements

### Requirement: Workspace tab session persistence

The system SHALL persist workspace-scoped tab session state and restore it when the workspace is reopened, subject to user settings.

#### Scenario: restore workspace tab session

- **WHEN** the user reopens a workspace and tab session restore is enabled
- **THEN** the previously open tabs are restored for that workspace
- **THEN** the active tab, pinned states, expanded directories, and tab groups are restored from the saved workspace session

#### Scenario: restore disabled by setting

- **WHEN** the user disables tab session restore and opens a workspace
- **THEN** the saved workspace tab session is not automatically applied
- **THEN** the workspace can still be opened normally

### Requirement: Workspace session payload is versioned and backward compatible

The system SHALL read legacy workspace tab session payloads and upgrade them to the new session model without breaking existing users.

#### Scenario: legacy session payload without version

- **WHEN** the system loads an older workspace tab session payload that contains only tabs and active index
- **THEN** the payload is interpreted as a legacy version
- **THEN** missing pinned and group fields are filled with default values

#### Scenario: new session payload is saved

- **WHEN** the system saves workspace tab session state in the new format
- **THEN** the payload includes an explicit version
- **THEN** the saved data is sufficient to restore grouped and pinned tabs for that workspace
