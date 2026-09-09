# Architecture Glossary

This glossary defines the terms used by BrotaParser. Definitions are operational: each term states how the compiler or analyzer may use it.

## Architecture

The set of significant structures, boundaries, relationships, and constraints that influence how a system is organized and changed.

In BrotaParser, architecture is not merely a folder layout. It may include:

- decomposition strategy;
- boundaries;
- dependency direction;
- visibility;
- import restrictions;
- public APIs;
- path rules;
- required artifacts;
- exceptions;
- invariants.

## Architectural style

A general family of principles and constraints.

Examples of profiles:

- Clean Architecture;
- Ports and Adapters;
- Vertical Slice Architecture;
- Feature-Sliced Design.

An architectural style is not automatically executable. BrotaParser requires a concrete profile before it can apply deterministic rules.

## Architecture profile

A versioned and executable configuration of architectural concepts, rules, defaults, and conventions.

Examples:

```text
    angular@0.1
clean-classic@0.1
vertical-slice@0.1
feature-sliced-design@2.1
```

A profile must define:

- purpose;
- concepts;
- defaults;
- mandatory rules;
- configurable rules;
- diagnostics;
- limitations;
- valid and invalid examples;
- source provenance.

## Architecture contract

A compilable declaration of the architecture that a project must follow.

Example rule fragment:

```brota
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
}
```

A source contract is compiled into a normalized `ArchitectureContract`.

## Decomposition

The primary criterion used to divide a system.

Common decomposition axes include:

- feature;
- business capability;
- use case;
- request;
- domain;
- bounded context;
- technical layer.

Decomposition is separate from internal structure. A project may be decomposed by feature and use layers inside every feature.

## Boundary

A logical border used to control membership, visibility, or dependencies.

A boundary may represent:

- feature;
- layer;
- slice;
- bounded context;
- module;
- library;
- application.

Different concepts may share structural properties without becoming semantically equivalent.

## Feature

A functional capability recognizable by users, the business, or the product.

Examples:

- authentication;
- catalog;
- checkout;
- user administration;
- reporting.

A feature is not automatically a DDD bounded context.

## Slice

A vertical unit organized around a request, action, use case, or behavior.

Examples:

- create order;
- cancel order;
- reset password;
- approve user.

A slice may contain every technical element required to complete that behavior.

## Layer

A division based on responsibility, abstraction level, or dependency policy.

Examples:

- domain;
- application;
- infrastructure;
- presentation.

A layer may define permitted dependencies, forbidden packages, required artifacts, and visibility constraints.

## Bounded context

A boundary within which a particular model and language have a defined meaning.

A bounded context is not automatically equivalent to:

- feature;
- Angular module;
- page;
- folder;
- microservice.

BrotaParser can validate declared boundaries. It cannot determine whether the business model is correct.

## Ubiquitous language

The shared domain vocabulary used by developers and domain experts inside a bounded context.

BrotaParser may validate naming conventions, but it cannot prove semantic correctness or business alignment.

## Port

A technology-independent interaction point between the application and an external actor.

Examples:

- repository contract;
- payment gateway;
- authentication provider;
- notification service;
- use-case input interface.

## Adapter

A technology-specific implementation that connects a port to an external mechanism.

Examples:

- HTTP client;
- Angular component;
- local storage implementation;
- WebSocket client;
- third-party SDK wrapper.

## Dependency

A relationship in which one element needs to know or use another element.

BrotaParser should distinguish:

- source-code dependency;
- static import;
- dynamic import;
- package dependency;
- boundary dependency;
- allowed dependency;
- forbidden dependency;
- required dependency.

## Dependency rule

A declaration that permits, prohibits, or requires a dependency.

Example:

```brota
allow show-times -> movie-reel
forbid checkout -> payments
```

The arrow direction means that the left side depends on the right side.

## Dependency cycle

A sequence of dependencies that returns to the original node.

Example:

```text
catalog -> orders -> payments -> catalog
```

Profiles must define the graph level at which cycles are checked and whether cycles are errors or warnings.

## Import rule

A restriction over source imports.

An import rule may use:

- origin boundary;
- destination boundary;
- package pattern;
- import type;
- public API requirement;
- exceptions.

## Public API

The authorized entry point of a boundary.

It may be represented by:

- `index.ts`;
- `public-api.ts`;
- package exports;
- aliases;
- manifests;
- framework configuration.

The architecture contract should describe the semantic requirement rather than assuming a single implementation mechanism.

## Deep import

An import that bypasses a boundary's public API and directly accesses internal implementation details.

## Ownership

The assignment of a resource to the boundary responsible for it.

Initial BrotaParser support should use ownership only for structural membership:

```text
file -> owning boundary
```

Team ownership and organizational governance should remain outside the initial compiler core.

## Root path

The base directory to which a contract or boundary applies.

## Path template

A path containing declared variables.

Example:

```text
src/app/features/{feature}/{layer}
```

Conformance analysis must ensure variables are known and resolve deterministically.

## Path rule

A rule that maps files or directories to architectural concepts.

## Required artifact

A file, directory, declaration, or export that must exist.

Profiles should avoid requiring ceremonial artifacts that do not enforce a meaningful property.

## Invariant

A condition that must always hold for a contract to be semantically valid.

Examples:

- identifiers are unique within their scope;
- references resolve;
- path variables are known;
- contradictory rules are rejected;
- normalized output is deterministic.

## Exception

An explicit and limited deviation from a rule.

Exceptions should declare scope and should not depend on textual order.

## Diagnostic

A structured description of a syntax, semantic, or conformance problem.

A diagnostic should contain:

- stable code;
- severity;
- message;
- primary span;
- labels;
- optional help.

## AST

The syntax-oriented representation of what the user wrote.

The AST preserves source structure and spans. It is not the public conformance contract.

## Normalized contract

The validated, resolved, deterministic representation of an architecture contract.

It should contain no unresolved references and should be serializable and comparable in tests.

## Project graph

A normalized representation of the analyzed source project.

It may include:

- files;
- modules;
- imports;
- packages;
- boundaries;
- symbols;
- ownership relations.

## Conformance rule

A rule that evaluates a `ProjectGraph` against an `ArchitectureContract`.

## Fitness function

An automated check that continuously evaluates an architectural property.

In BrotaParser, conformance rules act as structural fitness functions.
