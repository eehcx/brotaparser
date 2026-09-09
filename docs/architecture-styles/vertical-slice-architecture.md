# Vertical Slice Architecture

## Purpose

Vertical Slice Architecture organizes a system around requests, actions, or use cases rather than global technical layers.

Its main goals are:

- maximize cohesion within a slice;
- minimize coupling between slices;
- optimize for change by behavior.

## Slice semantics

A slice may contain everything needed for one behavior:

- UI;
- validation;
- application logic;
- data access;
- mapping;
- tests.

Different slices may use different implementation patterns.

## What Vertical Slice Architecture does not require

It does not inherently require:

- global domain/application/infrastructure/presentation layers;
- four internal layers in every slice;
- Clean Architecture inside every slice;
- tactical DDD;
- one shared service layer.

Imposing a uniform layered template on every slice can contradict the approach's intent.

## Frontend application

Possible frontend slices:

```text
create-order/
cancel-order/
reset-password/
approve-user/
```

A product feature may contain several slices.

## BrotaParser support

Potential checks:

- file ownership by slice;
- no forbidden cross-slice imports;
- no slice cycles;
- allowed shared dependencies;
- optional public APIs;
- required artifacts for declared behavior.

## Primary reference

Source registry ID: `vertical-slice-architecture`

Jimmy Bogard, *Vertical Slice Architecture*:

- https://www.jimmybogard.com/vertical-slice-architecture/
