# `angular`

## Identity

```text
Name: angular
Owner: BrotaParser
Category: framework organization profile
Status: initial profile
Profile version: 0.1
```

## Purpose

The `angular` profile captures Angular's project-organization guidance: group
related code by feature area, keep related files together, and avoid a global
structure organized only by technical file type.

This profile is a baseline. It does not claim that Angular defines Clean
Architecture, DDD, Feature-Sliced Design, or Vertical Slice Architecture.

## Contract dimensions

```text
decomposition: feature-areas
internal structure: angular-style-guide
dependency policy: framework-unspecified
visibility: framework-unspecified
```

Additional architecture profiles may add stricter dependency, visibility, or
domain-boundary rules without changing the Angular baseline.

## Generation authority

The versioned `angular@0.1` Blueprint is the source of truth for generators. It
defines the initial shell, support areas, feature areas, artifact identities,
and diagnostic rules. BrotaLibre or another downstream tool owns template
rendering and file writes; it must not replace the Blueprint with a local
architecture enum.

## References

- Source registry ID: `angular-style-guide`
- [Angular feature organization](../../architecture-styles/angular-feature-organization.md)
- [Version 0.1 specification](0.1.md)
- [Blueprints, contracts, and snapshots](../../concepts/blueprints.md)
- [Examples](examples.md)
