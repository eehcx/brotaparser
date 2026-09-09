# ADR-0003: Use Angular Organization as the Baseline Profile

**Status:** Accepted
**Date:** 2026-08-02

## Context

The repository initially modeled a local composition that was not a recognized
architecture. It mixed framework organization, Clean-inspired dependencies,
and product-specific conventions.

Angular already provides an official baseline for organizing frontend projects:
group code by feature areas, keep related files together, and avoid global
technical-type directories.

## Decision

The first supported profile is `angular@0.1`.

It intentionally remains a baseline profile. It does not invent Clean layers or
claim that Angular prescribes DDD, Feature-Sliced Design, or Vertical Slice
Architecture.

Future profiles may add recognized architectural constraints:

- Clean Architecture dependency direction.
- Ports and Adapters boundaries.
- Feature-Sliced Design layers and public APIs.
- Vertical Slice use-case decomposition.
- DDD strategic boundaries.

These profiles must document their own sources, scope, and verification level.

## Consequences

- The unsupported local composition is removed from the language, tests, fixtures,
  and public catalog.
- The contract core remains composable and does not become Angular-specific.
- BrotaParser publishes the `angular@0.1` Blueprint as the authority for the
  initial generated layout.
- BrotaLibre renders the Blueprint and collects project facts, but does not
  define the architecture.
- A derived `.brota` contract records the Blueprint provenance and is used for
  later conformance diagnostics.
- BrotaLibre can add a stricter profile when that architecture is intentionally
  selected.
- No released consumers require migration yet.
