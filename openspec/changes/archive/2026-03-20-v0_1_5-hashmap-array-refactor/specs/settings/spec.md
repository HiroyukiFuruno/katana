## ADDED Requirements

### Requirement: Type-safe Extensible Settings Lists
The domain model storing supplementary user configurations (`extra`) SHALL be represented strictly as a List of structurally typed configurations (`Vec<ExtraSetting>`).

#### Scenario: Persisting generic extensions
- **WHEN** application needs to store random key-value extras
- **THEN** it serializes to a JSON array of `{"key": "...", "value": "..."}` objects instead of an implicit Map `{ "key": "value" }`.

### Requirement: Automatic Format Migration
The application SHALL migrate legacy JSON objects into JSON arrays immediately upon load when encountering legacy `v0.1.3` (or older) settings structures.

#### Scenario: User upgrades from v0.1.3 to v0.1.4
- **WHEN** the application boots and discovers a legacy `settings.json` where `extra` is an object
- **THEN** the migration runner (`Migration0_1_3_to_0_1_4`) safely transforms it into the new Array schema before failing structural parsing.
