## MODIFIED Requirements

### Requirement: Diagram preview uses the current theme snapshot

The system SHALL render diagram previews from the current theme snapshot instead of an application-start snapshot or a global dark/light toggle alone.

#### Scenario: Mermaid preview follows a same-mode color change

- **WHEN** the user changes preview text color or related theme colors while staying in the same dark/light mode
- **THEN** Mermaid rendering uses the updated theme snapshot
- **THEN** the preview does not reuse a stale diagram image rendered with the previous color set

#### Scenario: PlantUML preview follows a same-mode color change

- **WHEN** the user changes preview text color or related theme colors while staying in the same dark/light mode
- **THEN** PlantUML rendering uses the updated theme snapshot
- **THEN** the preview does not reuse a stale diagram image rendered with the previous color set

### Requirement: Diagram cache keys are theme-aware

The system SHALL include the active diagram theme fingerprint in the persistent diagram cache key.

#### Scenario: Theme fingerprint changes

- **WHEN** the active diagram theme fingerprint changes for the same markdown file, diagram kind, and source
- **THEN** the cache key changes
- **THEN** the system re-renders the diagram instead of reusing the stale cached result
