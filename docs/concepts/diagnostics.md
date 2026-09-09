# Diagnostics and Verification Levels

## Diagnostic structure

Recommended representation:

```rust
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub context: Option<String>,
    pub locations: Vec<Location>,
}
```

## Stable codes

Consumers must depend on diagnostic codes, not human-readable messages.

Messages may improve without breaking integrations.

Recommended families:

```text
BRT-SYN-*  syntax
BRT-SEM-*  semantic compilation
BRT-PATH-* paths and ownership
BRT-DEP-*  dependencies
BRT-API-*  visibility and public APIs
BRT-FW-*   framework baseline profiles
```

The current compiler emits these stable codes:

```text
BRT-SYN-001  syntax error
BRT-PRO-001  unsupported language version
BRT-PRO-002  unsupported profile version
BRT-CON-001  missing contract declaration
BRT-CON-002  duplicate contract declaration
BRT-SEM-001  duplicate symbol
BRT-SEM-002  unresolved symbol
BRT-DEP-001  contradictory dependency rules
BRT-DEP-002  duplicate dependency rule
BRT-DEP-003  invalid exception
BRT-PATH-001 invalid path
BRT-PATH-002 invalid required artifact
BRT-PATH-003 duplicate path rule
BRT-PATH-004 ambiguous path ownership
BRT-PATH-005 duplicate required artifact
BRT-BLU-001 blueprint/contract incompatibility
BRT-CONF-001 project path ownership violation
BRT-CONF-002 missing required project artifact
BRT-CONF-003 project dependency conformance violation
BRT-CONF-004 unresolved project evidence
```

## Example

```text
error[BRT-DEP-002]: forbidden dependency

  --> src/app/features/catalog/domain/product.ts:3:1
   |
  3 | import { Something } from "./internal";
    | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = the selected profile forbids this dependency
```

## Severity

Recommended levels:

```rust
pub enum Severity {
    Error,
    Warning,
    Info,
}
```

An `unsupported` state should be represented separately from invalidity when possible.

## Verification levels

Every rule must declare its verification level.

### Deterministic

The analyzer can evaluate the rule objectively.

Examples:

- unknown reference;
- duplicate identifier;
- invalid dependency direction;
- deep import;
- cycle;
- missing required file;
- unknown path variable;
- forbidden package import.

Deterministic violations may produce errors.

### Heuristic

The analyzer can only estimate risk.

Examples:

- feature may be too large;
- layer may be ceremonial;
- code may be placed in the wrong layer;
- excessive use of shared code;
- probable infrastructure logic in domain.

Heuristic findings should normally produce warnings.

### Human review

The property cannot be established by static structure alone.

Examples:

- correct bounded-context boundary;
- quality of ubiquitous language;
- correctness of an aggregate;
- suitability of DDD;
- value of an abstraction;
- business cohesion of a feature.

These belong in documentation, ADRs, or review checklists.

## Source spans

The parser and AST must preserve spans for:

- declarations;
- identifiers;
- references;
- path variables during conformance analysis;
- dependency endpoints;
- contradictory rules.

Conformance diagnostics should point to project source files. Semantic diagnostics should point to `.brota` source locations.

## Fixture expectations

Invalid fixtures should verify:

- code;
- severity;
- primary span;
- key labels;
- stable structured data.

Avoid tests that only assert `is_err()`.

Example:

```rust
assert_eq!(
    diagnostic.code,
    "BRT-SEM-002"
);
```
