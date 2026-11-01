# ADR 0001: Own engine, not Unity and not a full existing engine

## Status

Accepted.

## Context

The project target is a Rust rewrite with an engine owned by the project. The original Unity/C# code contains hot-path patterns such as per-frame logging, repeated component lookup, object-name search, and large MonoBehaviour update methods. A line-by-line transliteration would preserve those architectural problems.

## Decision

Build a small custom engine layer in Rust:

- fixed-step simulation clock;
- ECS-based game state;
- stable asset IDs and data-only scenes;
- render backend trait with headless null backend first;
- explicit system stages and event queues;
- deterministic replay and benchmark harness.

Use small audited libraries where they do not surrender engine ownership: `hecs` for ECS storage, `glam` for math, `serde` for data, `tracing` for diagnostics. Evaluate `wgpu` for the first GPU backend.

## Consequences

Positive:

- no Unity runtime dependency;
- no GC pauses in simulation;
- testable headless core;
- clear performance budgets;
- portable backend boundary.

Negative:

- more engine work than using a full existing engine;
- renderer, animation, physics, UI, and tools must be built or integrated deliberately;
- requires strong CI and profiling discipline.
