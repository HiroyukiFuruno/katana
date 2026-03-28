## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: Workspace root can be opened as the application context
The system SHALL allow the user to choose a local directory and load it as the active workspace root for the Katana session.

#### Scenario: Open a valid workspace directory
- **WHEN** the user selects a readable local directory
- **THEN** the system loads that directory as the active workspace root
- **THEN** the workspace tree, editor context, and preview context are bound to that root

#### Scenario: Reject an unreadable workspace directory
- **WHEN** the user selects a directory that cannot be read
- **THEN** the system keeps the previous workspace unchanged
- **THEN** the user is shown a recoverable error state

### Requirement: Workspace tree reflects the files within the active project
The system SHALL display the active workspace as a navigable project tree that reflects files and directories from the workspace root.

#### Scenario: Render project contents after workspace load
- **WHEN** a workspace root has been loaded successfully
- **THEN** the system shows directories and files from that workspace in the project tree
- **THEN** the active document selection can be changed from that tree

#### Scenario: Open a document from the project tree
- **WHEN** the user selects a Markdown document in the project tree
- **THEN** the system loads that document into the editor
- **THEN** the preview pipeline uses that document as the active source

### Requirement: The shell layout preserves the MVP navigation model
The system SHALL present a desktop shell with dedicated areas for workspace navigation, document editing, and preview rendering.

#### Scenario: Show the default MVP layout
- **WHEN** the application starts with an active workspace
- **THEN** the user sees a workspace pane, an editor pane, and a preview pane
- **THEN** the shell reserves a consistent location for future menu and AI panel expansion
