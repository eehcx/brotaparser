# Compiler and Analyzer Model

## Processing pipeline

```text
SourceFile
    ↓
Pest parser
    ↓
Parse tree
    ↓
Owned AST
    ↓
Symbol collection
    ↓
Reference resolution
    ↓
Semantic validation
    ↓
ArchitectureBlueprint resolution
    ↓
Blueprint-to-contract derivation
    ↓
ArchitectureContract
```

Project analysis is a separate pipeline:

```text
Angular or Astro project
    ↓
Framework adapter
    ↓
ProjectSnapshot
```

Conformance analysis combines both:

```text
ArchitectureContract + ProjectSnapshot
                  ↓
          Conformance engine
                  ↓
              Diagnostics
```

## Parser responsibilities

The parser answers:

> Does the input have valid structure according to the language grammar?

It should validate local syntax such as:

- block structure;
- keywords;
- literals;
- punctuation;
- declaration shape.

It should not resolve names or architecture meaning.

## AST responsibilities

The AST represents what the user wrote.

It should preserve:

- declaration kind;
- original identifiers;
- source order;
- spans;
- explicit rules;
- unexpanded values.

Pest `Pair` values must not become the public compiler contract.

## Semantic compiler responsibilities

The semantic compiler should:

1. collect declarations;
2. build symbol tables;
3. resolve references;
4. detect duplicates;
5. validate path variables during contract compilation;
6. detect contradictory rules;
7. validate profile-specific invariants;
8. expand defaults;
9. produce normalized output.

## Normalized contract responsibilities

The normalized contract represents what BrotaParser understood.

It must be:

- deterministic;
- validated;
- resolved;
- serializable;
- order-independent where order has no semantics;
- suitable for conformance analysis.

## Project adapter responsibilities

A project adapter translates framework and language structures into a common
`ProjectSnapshot`. The adapter owns collection; the parser owns policy.

An Angular adapter may inspect:

- TypeScript files;
- imports;
- route declarations;
- workspace configuration;
- path aliases;
- package metadata.

An Astro adapter may inspect:

- pages;
- layouts;
- components;
- integrations;
- configuration;
- imports.

Adapters should avoid encoding architecture policy. They produce facts; profiles
decide whether those facts conform.

## Suggested logical modules

```text
src/
  syntax/
    grammar.pest
    parser.rs
    ast.rs
  semantics/
    symbols.rs
    resolver.rs
    validation.rs
    dependency_graph.rs
  contract/
    mod.rs
    normalization.rs
  diagnostics/
    mod.rs
    codes.rs
    renderer.rs
  blueprint.rs
  snapshot.rs
  conformance.rs
  source/
    span.rs
    source_file.rs
  project/
    graph.rs
    adapters/
```

These do not need to become separate crates immediately.
