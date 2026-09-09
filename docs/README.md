# BrotaParser Architecture Catalog

**Status:** Draft
**Catalog version:** 0.1.0
**Scope:** `.brota` language, architecture Blueprints, semantic compiler, contracts, snapshots, and conformance analysis

This documentation defines the architectural vocabulary and supported profiles of BrotaParser.

Its goals are to:

1. Keep syntax, semantics, architecture theory, and Brotalibre-specific decisions separate.
2. Avoid treating unrelated architectural approaches as interchangeable.
3. Define concepts in operational terms that can be compiled or validated.
4. Document the origin and verification level of every executable rule.
5. Provide a stable basis for adding new architecture profiles.

BrotaParser does not attempt to prove that a software architecture is appropriate for a business problem. It validates whether a project conforms to a declared and versioned architecture contract.

## Documentation map

### Core concepts

- [Glossary](concepts/glossary.md)
- [Boundaries and decomposition](concepts/boundaries.md)
- [Dependencies and visibility](concepts/dependencies.md)
- [Paths and required artifacts](concepts/paths.md)
- [Diagnostics and verification levels](concepts/diagnostics.md)
- [Compiler model](concepts/compiler-model.md)
- [Blueprints, contracts, and snapshots](concepts/blueprints.md)
- [Architecture contract JSON schema](schema/architecture-contract-0.1.schema.json)
- [Architecture Blueprint JSON schema](schema/architecture-blueprint-0.1.schema.json)
- [Project snapshot JSON schema](schema/project-snapshot-0.1.schema.json)

### Architectural approaches

- [Domain-Driven Design](architecture-styles/domain-driven-design.md)
- [Clean Architecture](architecture-styles/clean-architecture.md)
- [Ports and Adapters](architecture-styles/ports-and-adapters.md)
- [Vertical Slice Architecture](architecture-styles/vertical-slice-architecture.md)
- [Feature-Sliced Design](architecture-styles/feature-sliced-design.md)
- [Angular feature organization](architecture-styles/angular-feature-organization.md)

### Baseline profiles

- [`angular@0.1`](profiles/angular/README.md)
- [Specification](profiles/angular/0.1.md)
- [Examples](profiles/angular/examples.md)

The `angular` profile is the baseline for frontend projects. Clean Architecture,
Ports and Adapters, Feature-Sliced Design, Vertical Slice Architecture, and DDD
will be added only as explicit profiles or composable dimensions with their own
documented rules.

BrotaParser publishes the Blueprint that generators consume. Downstream tools
such as BrotaLibre render that Blueprint, collect project facts, and present
the diagnostics produced by the parser.

### Public governance

- [Source registry](sources/registry.toml)
- [ADR-0001: Versioned architecture profiles](adr/0001-versioned-architecture-profiles.md)
- [ADR-0003: Angular is the baseline profile](adr/0003-angular-baseline-profile.md)

### CLI

The CLI accepts a `.brota` file or stdin:

```bash
brotaparser contract.brota
cat contract.brota | brotaparser
```

Exit code `0` means the contract compiled successfully. Exit code `1` means
syntax or semantic diagnostics were emitted. Exit code `2` means the input
could not be read.

## Normative language

The following terms are used in profile specifications:

- **MUST:** mandatory requirement.
- **MUST NOT:** prohibited behavior.
- **SHOULD:** recommended behavior whose violation requires justification.
- **MAY:** optional behavior.
- **DEFAULT:** behavior applied when a contract does not provide another value.
- **EXPERIMENTAL:** behavior without a stable compatibility guarantee.

## Rule metadata

Every executable rule should declare:

```rust
pub struct RuleMetadata {
    pub id: String,
    pub origin: RuleOrigin,
    pub sources: Vec<String>,
    pub verification: VerificationLevel,
}
```

Recommended classifications:

```rust
pub enum RuleOrigin {
    CanonicalSource,
    FrontendAdaptation,
    FrameworkConvention,
    ProductDecision,
}

pub enum VerificationLevel {
    Deterministic,
    Heuristic,
    HumanReview,
}
```

## Compatibility

Architecture profiles must be versioned independently from the `.brota` language.

Example:

```brota
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
}
```

A language version controls syntax and general semantic capabilities. A profile version controls architecture-specific defaults and constraints.
