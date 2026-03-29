## MODIFIED Requirements

### Requirement: Theme changes propagate to diagram preview without restart

The system SHALL propagate runtime theme changes to diagram preview rendering without requiring an application restart.

#### Scenario: Theme mode changes

- **WHEN** the user switches between dark and light theme modes
- **THEN** diagram previews are refreshed using the newly active theme snapshot

#### Scenario: Preview text color changes

- **WHEN** the user changes preview-specific text color from the settings UI
- **THEN** diagram previews are refreshed using the new color values
- **THEN** the result matches the current preview theme instead of the previous one
