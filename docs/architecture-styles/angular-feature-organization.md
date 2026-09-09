# Angular Feature Organization

## Purpose

Angular's official style guidance recommends organizing projects by feature areas rather than globally grouping all files by technical type.

Preferred conceptual organization:

```text
src/app/
  catalog/
  checkout/
  users/
```

Less desirable as a global structure:

```text
src/app/
  components/
  services/
  directives/
```

## BrotaParser baseline

These recommendations are represented by the `angular@0.1` baseline profile.
The profile does not add domain, application, infrastructure, or presentation
layers because Angular's guide does not prescribe them.

## Relevance to BrotaParser

Angular guidance supports feature-oriented outer decomposition.

An Angular adapter may analyze:

- source files;
- imports;
- routes;
- standalone components;
- providers;
- workspace configuration;
- TypeScript path aliases.

The adapter should produce project facts. The selected architecture profile decides whether those facts are valid.

## Official reference

Angular Style Guide:

- Source registry ID: `angular-style-guide`
- https://angular.dev/style-guide
