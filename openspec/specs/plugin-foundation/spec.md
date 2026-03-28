## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: The core exposes typed plugin extension points
The system SHALL provide typed extension points for at least renderer enhancements, AI tools, and UI panel contributions.

#### Scenario: Register built-in plugin capabilities at startup
- **WHEN** the application initializes its extension registry
- **THEN** built-in plugins can register against the supported extension point types
- **THEN** the core can enumerate registered plugins by capability

### Requirement: Plugin loading for the MVP is controlled and versioned
The system SHALL limit MVP plugin loading to trusted bundled implementations that declare compatibility with the current plugin API contract.

#### Scenario: Accept a compatible bundled plugin
- **WHEN** a bundled plugin declares a compatible plugin API version
- **THEN** the plugin is registered during startup
- **THEN** its enabled capabilities become available to the application

#### Scenario: Startup uses static built-in registrations only
- **WHEN** the application initializes the MVP plugin registry
- **THEN** plugin registrations are resolved from built-in compile-time definitions
- **THEN** no runtime manifest file is required to activate bundled plugins

#### Scenario: Reject an incompatible bundled plugin
- **WHEN** a bundled plugin declares an incompatible plugin API version
- **THEN** the system does not activate that plugin
- **THEN** the application records the incompatibility without crashing core workflows

### Requirement: Plugin failures are isolated from core editing flows
The system MUST prevent a plugin initialization or execution failure from breaking the workspace, editor, or preview experience.

#### Scenario: Contain a plugin startup failure
- **WHEN** a plugin fails during initialization
- **THEN** the system disables that plugin instance
- **THEN** the application continues to load the workspace shell and Markdown authoring features
