# BrotaParser

BrotaParser is a Rust parser and compiler for declarative frontend architecture
contracts. It is independent and reusable, with BrotaLibre as its primary
consumer.

## Current status

The current public baseline is `angular@0.1`. It models Angular feature-area
organization and generic contract rules without claiming to implement Clean
Architecture, DDD, Feature-Sliced Design, or Vertical Slice Architecture.

## Quick start

Compile a contract:

```bash
cargo run -- contract.brota
```

Read from stdin:

```bash
cat contract.brota | cargo run --quiet
```

The normalized contract is written as JSON. See [`docs/cli.md`](docs/cli.md)
for exit codes and diagnostics.

## Language

The language supports versioned contracts, profiles, areas, dependency rules,
paths, required artifacts, exceptions, and comments:

```brota
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
  area catalog
  area orders
  allow orders -> catalog
}
```

Read the [architecture catalog](docs/README.md) for the language contract,
profile rules, sources, and compatibility policy.

## Library API

Downstream Rust applications can compile typed project configuration without
depending on the parser AST:

```rust
use brotaparser::{compile_profile, ProfileInput};

let input = ProfileInput {
    contract_name: "admin-panel".to_owned(),
    root: "src".to_owned(),
    areas: vec!["catalog".to_owned(), "orders".to_owned()],
    dependencies: Vec::new(),
    exceptions: Vec::new(),
    paths: Vec::new(),
    required_artifacts: Vec::new(),
};

let contract = compile_profile("angular", "0.1", &input)?;
```

Use `compile_source` when the input is already a `.brota` document. Use
`compile_profile` when a downstream application owns the project configuration
and wants a typed `ArchitectureContract` directly.

For architecture-authoritative generation, resolve the built-in Blueprint
instead. Blueprints describe the structure a generator must materialize, while
the derived `.brota` contract remains diagnostic:

```rust
use brotaparser::{derive_contract, render_contract_source, resolve_blueprint};

let blueprint = resolve_blueprint("angular", "0.1")?;
let contract = derive_contract(&blueprint, "admin-panel");
let diagnostic_source = render_contract_source(&contract);
```

`compile_profile` remains available as a compatibility API for consumer-defined
contracts; it is not the authority API for built-in architecture profiles.

## Development

```bash
cargo fmt -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

BrotaParser compiles architecture contracts. Filesystem, TypeScript, Angular,
and project conformance analysis belong to downstream adapters such as
BrotaLibre.

## Publishing

Releases are published automatically when an immutable
`v<major>.<minor>.<patch>` Git tag is pushed. The tag version must match
`Cargo.toml`.

Before the first release, create a crates.io API token and add it to the
repository Actions secrets as `CRATES_IO_TOKEN`. Then:

```bash
cargo fmt -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo publish --dry-run --locked
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

The `Publish crate` workflow verifies the tag, package version, and package
contents before publishing to crates.io.
