# Ports and Adapters

## Purpose

Ports and Adapters keeps application behavior independent from external interfaces, devices, and technologies.

It is also commonly called Hexagonal Architecture.

## Core concepts

- application inside;
- ports;
- adapters;
- primary actors;
- secondary actors;
- replaceable external technology.

## Ports

A port represents a purposeful conversation with the application.

Frontend examples:

- use-case input;
- product repository;
- authentication provider;
- payment gateway;
- notification channel.

## Adapters

An adapter translates between a port and a concrete technology.

Frontend examples:

- Angular component invoking an input port;
- HTTP repository implementing an output port;
- `localStorage` persistence adapter;
- WebSocket event adapter;
- external SDK wrapper.

## Relation to Clean Architecture

Both approaches isolate policy from details. Ports and Adapters emphasizes interaction points and replaceable adapters. Clean Architecture emphasizes dependency direction and policy levels.

They can be combined, but should not be treated as identical terminology.

## BrotaParser support

Potential checks:

- port declarations exist;
- adapters implement or reference known ports;
- domain/application code does not depend on concrete adapters;
- adapter placement follows profile rules;
- technologies are restricted to outer boundaries.

## Primary reference

Source registry ID: `hexagonal-architecture`

Alistair Cockburn, *Hexagonal Architecture*:

- https://alistair.cockburn.us/hexagonal-architecture
