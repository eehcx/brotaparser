# Boundaries and Decomposition

## Purpose

This document defines how BrotaParser represents organizational units without collapsing distinct architectural concepts into one generic abstraction.

## Decomposition strategy

The decomposition strategy answers:

> What is the primary unit by which the system is divided?

Examples:

```rust
pub enum DecompositionStrategy {
    Feature,
    UseCase,
    Domain,
    BoundedContext,
    TechnicalLayer,
}
```

The decomposition strategy does not fully define the internal structure of each unit.

Examples:

```text
feature decomposition + clean internal layers
use-case decomposition + flat internal structure
bounded-context decomposition + feature substructure
```

## Boundary model

A normalized boundary may share common structural data:

```rust
pub struct CompiledBoundary {
    pub id: BoundaryId,
    pub kind: BoundaryKind,
    pub parent: Option<BoundaryId>,
    pub path_selector: PathSelector,
    pub visibility: VisibilityPolicy,
}
```

Recommended kinds:

```rust
pub enum BoundaryKind {
    Application,
    Library,
    BoundedContext,
    Feature,
    Slice,
    Layer,
    Module,
}
```

`BoundaryKind` must remain available after normalization. Rules may apply different semantics to features, layers, slices, and bounded contexts.

## Semantic distinctions

### Feature

A product or business-facing capability.

A feature may contain layers or slices.

### Slice

A request- or use-case-oriented unit of change.

A slice may be self-contained and does not require globally uniform internal layers.

### Layer

A responsibility or abstraction boundary with a dependency policy.

Layers often participate in directional rules.

### Bounded context

A model and language boundary from DDD.

A bounded context may contain several features and must not be inferred solely from folder names.

## Scope and uniqueness

Identifiers should be unique within their declared scope.

Conceptual boundary model (not surface syntax):

```text
feature catalog {
  layer domain
  layer application
}
```

The identifier `domain` may be reusable inside another feature if the language defines layer declarations as feature-scoped.

The compiler must define scope explicitly rather than applying global uniqueness to every name.

## Parent-child relationships

Boundaries may be nested:

```text
application
└── bounded context: sales
    ├── feature: orders
    │   ├── layer: domain
    │   └── layer: application
    └── feature: checkout
```

A profile must define which nesting combinations are legal.

## Ownership resolution

A file should resolve to at most one owner at a given semantic level.

For example, a file may simultaneously belong to:

- application `admin-panel`;
- bounded context `sales`;
- feature `orders`;
- layer `domain`.

It should not belong to two features unless the profile explicitly supports overlapping ownership.

## Public APIs

A boundary may declare a public API. Consumers outside that boundary must use it when the profile requires encapsulation.

The normalized contract should describe:

- whether a public API is required;
- the authorized entry points;
- which consumers are subject to the restriction;
- whether internal files may bypass the public API.

## Non-goals

The boundary model does not determine:

- whether a feature is well designed;
- whether a bounded context is correct;
- whether a folder should become a separate package;
- whether a module has sufficient cohesion.

Those require human architecture decisions or heuristic tooling.
