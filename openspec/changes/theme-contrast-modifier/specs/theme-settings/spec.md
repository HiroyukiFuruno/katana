## ADDED Requirements

### Requirement: UI Contrast Offset (Alpha Multiplier)
The system SHALL allow users to globally offset the alpha channel (opacity) of all semitransparent RGBa UI elements, ensuring maximum readability regardless of the selected theme preset.

#### Scenario: Adjusting Global Contrast
- **WHEN** the user modifies the "UI Contrast Adjustment" slider (`-100%` to `+100%`) in the Settings window
- **THEN** the alpha values of all relevant theme layers (e.g. `active_file_highlight`) MUST dynamically update according to the formula `new_alpha = clamp(original_alpha + 255 * (offset / 100.0), 0, 255)`

#### Scenario: Value Persistence
- **WHEN** the contrast adjustment is changed
- **THEN** the offset value MUST be persisted to `AppearanceSettings` as `ui_contrast_offset` and properly restored across application restarts
