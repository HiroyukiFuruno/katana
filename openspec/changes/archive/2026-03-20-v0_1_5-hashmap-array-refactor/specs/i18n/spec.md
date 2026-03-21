## ADDED Requirements

### Requirement: Deterministic Dictionary Initialization
The internationalization dictionary memory store SHALL iterate over its language map deterministically by using a List (`Vec<I18nDictionaryEntry>`) structure under `OnceLock`.

#### Scenario: Dictionary access mechanism
- **WHEN** the system queries for UI localization strings
- **THEN** it iterates over a continuous array memory space, retrieving the translation without structural ambiguity.
