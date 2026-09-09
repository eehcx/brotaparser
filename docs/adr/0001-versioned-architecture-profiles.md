# ADR-0001: Represent Architectures as Versioned Profiles

**Status:** Proposed
**Date:** 2026-08-02

## Context

DDD, Clean Architecture, Ports and Adapters, Vertical Slice Architecture, and Feature-Sliced Design answer different architectural questions.

Representing them as interchangeable enum values would imply that they are mutually exclusive and structurally equivalent.

BrotaParser also needs stable, executable rules. General architecture styles are too abstract to compile directly without profile-specific decisions.

## Decision

BrotaParser will represent executable architectures as versioned profiles.

A profile may combine several dimensions:

- decomposition strategy;
- boundary types;
- internal structure;
- dependency policy;
- visibility policy;
- path policy;
- required artifacts.

Example:

```text
angular@0.1
  decomposition = feature areas
  internal structure = Angular Style Guide
  dependency policy = framework-unspecified
```

The `.brota` language version and architecture profile version are independent.

## Consequences

### Positive

- architecture styles retain their real distinctions;
- Brotalibre decisions become explicit;
- rules can evolve under semantic versioning;
- users can pin behavior;
- additional profiles can reuse common primitives.

### Negative

- profile metadata and migration policy are required;
- the compiler must separate general language semantics from profile expansion;
- documentation volume increases.

## Rejected alternative

```rust
enum ArchitectureKind {
    Clean,
    VerticalSlice,
    FeatureSlicedDesign,
    DomainDrivenDesign,
}
```

This model was rejected because these concepts can coexist and do not describe the same architectural dimension.
