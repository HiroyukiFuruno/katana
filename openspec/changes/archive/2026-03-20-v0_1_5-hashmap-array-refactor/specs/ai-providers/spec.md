## ADDED Requirements

### Requirement: Provider Request Data Safety
AI request properties like `params` and `metadata` SHALL be modeled securely as explicit Lists of structures (e.g., `Vec<AiParam>`) instead of dynamic key-value pools.

#### Scenario: Sending an AI prompt
- **WHEN** the system generates an `AiRequest` and sets extra capabilities
- **THEN** parameters are appended sequentially into a strictly defined Array instead of being hashed dynamically.
