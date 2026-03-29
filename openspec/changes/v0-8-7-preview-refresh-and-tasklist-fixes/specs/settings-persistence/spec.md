## ADDED Requirements

### Requirement: Document auto-refresh behavior settings are persisted

The system SHALL persist the document auto-refresh enable flag and refresh interval as part of application behavior settings, and SHALL restore those values on the next launch.

#### Scenario: Save auto-refresh settings

- **WHEN** the user changes the document auto-refresh enable flag or refresh interval in settings
- **THEN** the updated values are saved to the application settings store
- **THEN** the next launch restores the same values

#### Scenario: Apply default auto-refresh settings

- **WHEN** the settings file does not yet contain document auto-refresh configuration
- **THEN** the application applies the agreed default enable flag and default interval
- **THEN** those defaults are exposed to the user through the settings UI

#### Scenario: Invalid auto-refresh setting is encountered

- **WHEN** the persisted document auto-refresh interval is missing, malformed, or outside the supported range
- **THEN** the application falls back to the defined default interval
- **THEN** the rest of the settings payload remains recoverable
