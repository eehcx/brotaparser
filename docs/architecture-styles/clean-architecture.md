# Clean Architecture

## Purpose

Clean Architecture separates business policies and use cases from external details while controlling source-code dependency direction.

## Dependency Rule

The central rule is:

> Source-code dependencies point toward higher-level policies.

Internal policy should not depend on UI frameworks, databases, delivery mechanisms, or other replaceable details.

## Conceptual areas

The original model commonly discusses:

- entities or enterprise business rules;
- use cases or application business rules;
- interface adapters;
- frameworks and drivers.

These areas are illustrative. Clean Architecture does not require exactly four folders or those exact names.

## Frontend interpretation

Possible frontend mapping:

- domain policy;
- application use cases;
- adapters and data access;
- presentation and framework integration.

Framework-specific packages such as Angular may be treated as outer details.

The exact package restrictions remain profile-specific decisions.

## What Clean Architecture does not prescribe

It does not inherently require:

- organization by feature;
- DDD tactical patterns;
- `index.ts` public APIs;
- four mandatory directories;
- identical internal structure for every feature.

## BrotaParser support

Potential deterministic rules:

- dependency direction;
- framework restrictions by layer;
- package restrictions;
- boundary crossings;
- adapter placement;
- forbidden imports.

## Primary reference

Source registry ID: `clean-architecture-2012`

Robert C. Martin, *The Clean Architecture*:

- https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html
