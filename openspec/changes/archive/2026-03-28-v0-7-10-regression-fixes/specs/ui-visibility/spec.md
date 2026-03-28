## MODIFIED Requirements

### Requirement: Apply Contrast Offset

- **Description:** Implement generic alpha tracking for all `Rgba` instances. The UI MUST use `with_contrast_offset` to reflect the user's `ui_contrast_offset` parameter.
- **Priority:** Must

#### Scenario: User adjusts UI contrast offset slider

- When a user increases the UI contrast offset in settings
- Then transparent elements (like hover lines) adjust their opacity appropriately
- And the change is immediately previewable.
