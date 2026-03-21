## ADDED Requirements

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
