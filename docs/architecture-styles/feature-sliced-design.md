# Feature-Sliced Design

## Purpose

Feature-Sliced Design is a frontend-oriented methodology based on layers, slices, segments, and public APIs.

## Main concepts

- layers;
- slices;
- segments;
- public API;
- import rule;
- slice isolation.

Common layers include:

- app;
- pages;
- widgets;
- features;
- entities;
- shared.

## Dependency direction

Slices may import from lower layers according to the methodology's import rule.

External consumers should access a slice through its public API rather than internal implementation paths.

## Relation to the baseline

Feature-Sliced Design is a stricter frontend architecture profile than the
baseline `angular@0.1` profile. It adds layers, slices, segments, public APIs,
and slice-isolation rules. It should be implemented as its own versioned profile
rather than being silently folded into Angular organization.

## Versioning relevance

Feature-Sliced Design has evolved over time. This reinforces the need for BrotaParser profiles to pin versions and avoid silently changing rule meaning.

## BrotaParser support

Potential profile rules:

- known layer ordering;
- same-layer slice isolation;
- public API enforcement;
- forbidden deep imports;
- segment conventions;
- migration diagnostics.

## Official documentation

Source registry ID: `feature-sliced-design`

- https://feature-sliced.design/docs/get-started/overview
- https://feature-sliced.design/docs/reference/layers
- https://feature-sliced.design/docs/reference/public-api
