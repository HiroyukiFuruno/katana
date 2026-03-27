## ADDED Requirements

### Requirement: Cache Retention Settings
The system SHALL allow users to configure the caching strategy and maximum retention period for HTTP downloaded images (e.g., Markdown badges) from the Settings window.

#### Scenario: Configuring Cache Strategy
- **WHEN** the user opens the System Settings section
- **THEN** an option to define the image cache lifetime MUST be visible (e.g., 7 days, 30 days, or unlimited)

### Requirement: Manual Cache Clearance
The system SHALL provide a direct mechanism to immediately purge all persisted HTTP image cache data to resolve potential disk space or caching state corruption issues.

#### Scenario: Resolving Directory Lock Errors
- **WHEN** the user clicks "Clear HTTP Image Cache" from the Settings UI
- **THEN** the system MUST safely attempt to delete all items inside `/Library/Caches/KatanA/http-image-cache/` ignoring strict directory locks (`os error 66`) by relying on iterative file deletion and graceful fallbacks, ensuring the application does not crash.
