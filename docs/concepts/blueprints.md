# Blueprints, Contracts, and Snapshots

BrotaParser separates architecture generation from architecture diagnosis.

## Blueprint

A versioned Blueprint is the prescriptive output of an architecture profile. It
describes the layout, areas, artifact identities, path ownership, dependency
rules, and stable template references that a downstream generator can
materialize.

The Blueprint does not contain executable templates. BrotaLibre owns its Angular
rendering tools and resolves the Blueprint's opaque `template_ref` values.

The first supported Blueprint is `angular@0.1`. Its feature-area set is
project-specific. The built-in resolver keeps `catalog`, `checkout`, and
`users` as a deterministic example set, while downstream generators can
resolve the same profile with their discovered areas:

```text
src/app/
  app.config.ts
  app.routes.ts
  app.ts
  core/
  shared/
  catalog/
  checkout/
  users/
```

The baseline uses feature areas and co-location. It does not generate global
technical-type directories or Clean Architecture layers.

## Diagnostic contract

The existing `.brota` language remains a declarative diagnostic format. A
Blueprint can derive an `ArchitectureContract` and a deterministic `.brota`
source document. The source document records areas, dependencies, paths,
required artifacts, forbidden paths, and exceptions, but never contains
templates or executable instructions.

Consumers can resolve and derive a contract through the library API:

```rust
use brotaparser::{derive_contract, resolve_blueprint, render_contract_source};

let blueprint = resolve_blueprint("angular", "0.1")?;
let contract = derive_contract(&blueprint, "admin-panel");
let brota_source = render_contract_source(&contract);
```

## Project snapshot

BrotaLibre remains responsible for collecting facts from the filesystem and
TypeScript analyzer. It submits a normalized `ProjectSnapshot` containing files,
directories, imports, symbols, routes, providers, configuration facts, and
evidence levels.

The parser evaluates the snapshot against the derived contract:

```rust
let diagnostics = brotaparser::diagnose(&contract, &snapshot);
```

Observed, inferred, and unresolved evidence remain distinct. Unresolved facts
that are needed by a rule produce an explicit diagnostic instead of silently
passing.

## Versioning

Blueprint and contract schemas are versioned independently. A derived contract
records the Blueprint identity and schema. Diagnosis checks this provenance
before evaluating project facts, preventing silent cross-version validation.
