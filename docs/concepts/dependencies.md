# Dependencies and Visibility

## Direction

Dependency direction must always be explicit.

```text
presentation -> application
```

means:

> `presentation` may depend on `application`.

It does not imply the reverse relationship.

## Rule kinds

BrotaParser should initially support:

```rust
pub enum DependencyRuleKind {
    Allow,
    Forbid,
    Require,
}
```

Example DSL:

```brota
allow application -> domain
forbid domain -> infrastructure
require infrastructure -> application
```

## Declarative semantics

Rule meaning must not depend on source order.

This is invalid:

```brota
allow application -> infrastructure
forbid application -> infrastructure
```

The compiler should emit a contradiction diagnostic rather than choosing the last rule.

The normalized dependency semantics are:

| Combination | Meaning |
|-------------|---------|
| `allow` + `allow` | Duplicate rule |
| `allow` + `require` | Compatible: permitted and required |
| `allow` + `forbid` | Contradiction |
| `require` + `forbid` | Contradiction |
| `forbid` + exception | Valid only when the exception is explicit and justified |

## Specificity

The language must define how broad and narrow rules interact.

Example:

```brota
allow application -> *
forbid application -> infrastructure
```

A recommended policy is:

1. A specific prohibition may restrict a broader permission.
2. Equivalent rules with opposite effects are contradictory.
3. Textual order never changes the result.
4. Exceptions are represented explicitly rather than through ordering.

## Dependency graph

Normalized rules should compile to a graph-oriented policy.

```rust
pub struct DependencyPolicy {
    pub allowed_edges: EdgeSet,
    pub forbidden_edges: EdgeSet,
    pub required_edges: EdgeSet,
}
```

The compiler may use adjacency sets, matrices, or another deterministic representation.

## Cycles

Profiles must define:

- graph node type;
- edge type;
- scope;
- severity;
- allowed internal cycles;
- exceptions.

Examples of distinct checks:

- cycle between features;
- cycle between slices;
- cycle between packages;
- cycle between files;
- cycle between conceptual layers.

These must not be treated as the same rule.

## Import categories

Project adapters should distinguish, where possible:

- static imports;
- dynamic imports;
- type-only imports;
- package imports;
- relative imports;
- absolute imports;
- framework metadata references.

Profiles may choose which categories count as architecture dependencies.

## Visibility policies

Recommended normalized forms:

```rust
pub enum VisibilityPolicy {
    Unrestricted,
    PublicApiRequired,
    ExplicitExports,
    PackageExports,
}
```

A profile should state whether the policy applies:

- across features;
- across slices;
- across libraries;
- across applications;
- inside the same boundary.

## Deep imports

A deep import crosses a boundary without using an authorized public entry point.

Example:

```ts
import { ProductRepository } from
  "@features/catalog/infrastructure/product.repository";
```

A profile may permit internal deep imports while forbidding external ones.

## Framework dependencies

A profile may restrict framework packages by layer.

Example:

```text
domain MUST NOT import @angular/**
```

Such rules must be classified correctly:

- dependency inversion principles may come from Clean Architecture;
- the exact package pattern is a frontend adaptation or Brotalibre decision.

## Exceptions

Recommended conceptual form:

```brota
exception allow orders -> catalog {
  through public-api
  reason "Orders displays catalog summaries"
}
```

An exception should be narrow, explicit, and independently diagnosable.

An exception does not silently delete the original rule. It remains visible in
the normalized contract so a conformance engine can evaluate the exception's
channel and reason.
