## MODIFIED Requirements

### Requirement: Heading-Based Synchronization

- **Description:** The UI MUST implement AST tracking (heading anchors) to perfectly align Markdown editor split views rather than relying on absolute document height percentages.
- **Priority:** Must

#### Scenario: User scrolls editor past heading boundaries

- When a user scrolls the markdown source pane downward crossing a level 2 heading
- Then the preview pane synchronizes to the identical proportional displacement between bounding headings rendering precision preview locking.
