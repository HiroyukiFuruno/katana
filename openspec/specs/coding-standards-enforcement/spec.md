## Purpose
This is a legacy capability specification that was automatically migrated to comply with the new OpenSpec schema validation rules. Please update this document manually if more context is required.

## Requirements

### Requirement: AST Linter execution on test suite
The system SHALL systematically parse the project's source code into Abstract Syntax Trees (AST) using `syn` during the execution of standard unit tests (`cargo test`).

#### Scenario: Running test suite triggers linter
- **WHEN** the `cargo test` command is executed in CI or via lefthook
- **THEN** the AST linter module (`katana-linter` crate) scans all `.rs` files across all workspace crates (`katana-core`, `katana-platform`, `katana-ui`)
- **THEN** it completes the scan and reports any coding standards violations by failing the test

### Requirement: Hardcoded i18n strings prevention (AST level)
The system SHALL detect and prevent string literals from being passed directly to defined UI rendering functions (like `ui.label()`, `ui.heading()`, `ui.button()`, `RichText::new()`, etc).

#### Scenario: Developer attempts to hardcode a string in a UI function
- **WHEN** the scanned source code contains a direct string literal such as `ui.button("Save")`
- **THEN** the Linter flags this AST node as a violation
- **THEN** the overall test suite fails and indicates the file and line number of the failure

### Requirement: Allowlist for non-alphabetical GUI symbols
The system SHALL bypass the hardcoded string prevention rule if the string literal consists explicitly of allowed characters (e.g., ascii symbols, singular emojis, whitespace).

#### Scenario: Developer uses purely symbolic text in UI function
- **WHEN** the scanned source code contains a string literal consisting only of symbols (e.g., `ui.label("/")` or `ui.button("▶")`)
- **THEN** the Linter matches this against the Allowlist and permits it
- **THEN** the test suite succeeds without reporting a violation

### Requirement: Magic number detection
The system SHALL detect numeric literals used outside of `const` or `static` declarations that are not in the permitted set (0, 1, -1, 2, 100).

#### Scenario: Developer uses an unnamed numeric literal
- **WHEN** the scanned source code contains a numeric literal such as `let width = 400.0;` outside a `const` declaration
- **THEN** the Linter flags this as a magic number violation
- **THEN** the test suite fails and indicates the file and line number

#### Scenario: Numeric literals in test code are excluded
- **WHEN** a numeric literal appears inside a `#[cfg(test)]` block (module, function, or impl method)
- **THEN** the Linter skips detection to avoid false positives on test-specific values

### Requirement: Language selector scalability
The system SHALL manage supported language definitions in a standalone data file (`locales/languages.json`), not in locale-specific translation JSON files or hardcoded in Rust source.

#### Scenario: Adding a new language
- **WHEN** a developer wants to add Chinese support
- **THEN** they add `{"code": "zh", "name": "中文"}` to `languages.json` and create `zh.json`
- **THEN** the language selector UI automatically includes the new language without any Rust code changes

### Requirement: HashMap usage is strictly forbidden
The system (AST Linter) SHALL emit an error `MDxxx` whenever the identifier `HashMap` (or `std::collections::HashMap`) is used for variable binding, structural type definition, or expression logic.

#### Scenario: Code contains HashMap
- **WHEN** user writes structural definitions using `HashMap` and runs lint
- **THEN** the linter emits a violation instructing them to use a `Vec<DataClass>` (List).

### Requirement: Primitive Arrays are strictly forbidden
The system (AST Linter) SHALL emit an error whenever `syn::Type::Array` (`[T; N]`) or `syn::Expr::Array` (`[a, b]`) is detected.

#### Scenario: Code contains an array literal
- **WHEN** user writes code like `[1, 2, 3]` instead of `vec![1, 2, 3]`
- **THEN** the linter emits a violation instructing them to use `Vec<T>`.
