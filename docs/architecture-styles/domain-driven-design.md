# Domain-Driven Design

## Purpose

Domain-Driven Design addresses complex software by aligning models, language, and boundaries with the business domain.

## Primary concepts

- domain;
- subdomain;
- ubiquitous language;
- bounded context;
- context map;
- entity;
- value object;
- aggregate;
- domain service;
- repository;
- domain event.

## What DDD does not prescribe

DDD does not inherently require:

- four folders;
- Angular;
- feature-based organization;
- vertical slices;
- microservices;
- a folder named `entities`;
- tactical patterns in every part of the system.

## Frontend application

Frontend systems may use DDD concepts to:

- define domain terminology;
- separate bounded contexts;
- isolate models;
- prevent cross-context imports;
- translate backend models through adapters or BFFs.

A frontend feature is not automatically a bounded context.

A UI flow may combine information from several backend contexts. The frontend may need its own view-specific model while preserving domain terminology where appropriate.

## BrotaParser support

Deterministic support may include:

- declared bounded contexts;
- allowed context relationships;
- public APIs between contexts;
- path ownership;
- import boundaries.

Human review remains necessary for:

- context correctness;
- model quality;
- aggregate design;
- ubiquitous language quality;
- domain significance.

## Primary reference

Source registry ID: `ddd-reference-2015`

Eric Evans, *Domain-Driven Design Reference*:

- https://www.domainlanguage.com/ddd/
- https://www.domainlanguage.com/wp-content/uploads/2016/05/DDD_Reference_2015-03.pdf
