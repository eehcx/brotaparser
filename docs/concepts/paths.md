# Paths and Required Artifacts

## Root paths

A root path defines the base location to which a contract applies.

```brota
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src/app"
}
```

The project adapter should normalize separators and resolve the path relative to
the workspace root. The contract compiler preserves the declared pattern.

## Path templates

A path template contains variables:

```brota
path "src/app/{area}/**" owns area catalog
```

The current compiler preserves path templates in the normalized contract.
Variable declaration and workspace matching belong to the conformance phase and
must be supplied by the selected profile or project adapter.

## Path variables

Recommended normalized representation:

```rust
pub struct PathVariable {
    pub name: Identifier,
    pub kind: PathVariableKind,
    pub span: Span,
}
```

Possible kinds:

```rust
pub enum PathVariableKind {
    Feature,
    Layer,
    Slice,
    BoundedContext,
    Custom,
}
```

## Path matching

Path rules should define:

- base path;
- recursive or exact matching;
- case sensitivity;
- separator normalization;
- excluded paths;
- ownership assignment.

Example:

```brota
path "src/app/catalog/**" owns area catalog
```

Blueprint-derived contracts can also reject architectural buckets:

```brota
forbid path "src/app/components/**"
forbid path "src/app/domain/**"
```

## Ambiguity

A project path should not resolve to conflicting owners at the same semantic level.

If two path rules assign one file to different features, the analyzer should emit an ambiguity diagnostic.

## Required artifacts

A required artifact can be:

- file;
- directory;
- public API;
- export;
- declaration;
- configuration entry.

Example:

```brota
require directory "src/app/catalog" in area catalog
require public-api "src/app/catalog/index.ts" in area catalog
```

## Conditional artifacts

A required artifact may depend on another declaration.

Example:

```text
If a feature declares infrastructure adapters,
it MUST declare at least one application port.
```

Such rules belong to semantic or conformance validation, not to Pest grammar.

## Avoiding ceremony

Profiles should not require a file or folder solely because an architecture diagram contains a conceptual category.

Examples:

- A simple feature may not need domain entities.
- A feature with no external integration may not need infrastructure.
- A slice may not need separate command and handler files.

Required artifacts must enforce a real architectural property.

## Framework adapters

The contract should describe semantic intent. Framework adapters resolve concrete mechanisms.

The compiler rejects empty, absolute, and traversal paths. Project conformance
adds workspace containment, glob matching, forbidden-path checks, and filesystem
ownership checks.

For example, `public-api` may map to:

- `index.ts`;
- `public-api.ts`;
- package exports;
- a generated manifest.

This keeps the architecture contract independent from a single file convention.
