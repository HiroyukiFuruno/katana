## ADDED Requirements

### Requirement: Users can organize tabs into named color groups

The system SHALL allow users to organize open tabs into named, color-coded groups.

#### Scenario: create a new group from a tab

- **WHEN** the user opens a tab context menu and selects the create-group action
- **THEN** a new tab group is created with a name, a color, and the selected tab as its first member
- **THEN** the selected tab belongs to at most one group

#### Scenario: add a tab to an existing group

- **WHEN** the user selects an existing group from the tab context menu
- **THEN** the selected tab becomes a member of that group
- **THEN** the tab is removed from any previous group membership

#### Scenario: pinned tab is not grouped

- **WHEN** a tab is pinned
- **THEN** it is not a member of any tab group
- **THEN** group-add actions are not offered or are disabled for that tab

#### Scenario: collapse a tab group

- **WHEN** the user collapses a tab group
- **THEN** member tabs are hidden from the tab bar without being closed
- **THEN** the group header remains visible and can be expanded again

#### Scenario: collapsed group keeps active member visible

- **WHEN** the active tab belongs to a collapsed group
- **THEN** the group header remains visible
- **THEN** the active member tab remains visible while non-active members are hidden

### Requirement: Pinned tabs are protected from normal close actions

The system SHALL protect pinned tabs from ordinary close affordances until they are explicitly unpinned.

#### Scenario: pinned tab hides close button

- **WHEN** a tab is pinned
- **THEN** its close button is not shown in the tab bar
- **THEN** the tab still exposes its title through a tooltip or equivalent affordance

#### Scenario: batch close skips pinned tabs

- **WHEN** the user triggers close-all, close-others, close-left, or close-right
- **THEN** pinned tabs are not closed
- **THEN** unpinned tabs continue to follow the requested close behavior

#### Scenario: shortcut close does not close pinned tab

- **WHEN** the active tab is pinned and a normal close shortcut dispatches a close action
- **THEN** the pinned tab remains open
- **THEN** the user must unpin it before it can be normally closed
