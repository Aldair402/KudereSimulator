# Architecture

## Principle

Rewrite the game as a deterministic, data-oriented Rust application with an engine owned by this project. Do not transliterate Unity/C# line by line. Preserve observable behavior and original identifiers as reference metadata, then implement idiomatic Rust systems.

## Crate graph

```text
kud_core
   |
   +--> kud_ecs
   |       |
   |       +--> kud_sim
   |
   +--> kud_asset
   +--> kud_render
              |
              +--> kud_engine --> kud_runner
```

- `kud_core`: fixed-step clock, errors, shared primitives.
- `kud_ecs`: thin facade over `hecs`; no gameplay logic.
- `kud_asset`: stable asset IDs, manifests, registry.
- `kud_render`: backend trait, camera/draw descriptions, null renderer.
- `kud_sim`: school simulation components and systems.
- `kud_engine`: frame loop, fixed-step orchestration, render submission.
- `kud_runner`: headless CLI for tests, benchmarks, and CI.

## Frame loop

1. Collect platform input into tick-stamped commands.
2. Add variable frame delta to `FixedClock`.
3. Run zero or more fixed simulation ticks.
4. Each tick runs explicit stages:
   - input/command application
   - school clock and schedule selection
   - AI state transitions
   - movement/navigation
   - perception and interaction queries
   - event emission
5. Drain events into gameplay systems.
6. Prepare animation/render state.
7. Submit render frame using interpolation alpha.

## Determinism rules

- Simulation uses fixed `f32` delta from `FixedStep`.
- Event ordering is stable and sorted by student ID/event kind.
- Randomness must use explicit seeded RNG streams, never global random.
- Replay stores input commands, seed, tick count, and state hashes.
- Render time must not feed back into simulation state.

## Data model

- Entities are generational ECS handles.
- Original student IDs are stable domain IDs mapped to entities.
- Assets use stable `u64` IDs, not object names.
- Schedules are data blocks with start/end minute, destination, action, and phase.
- Save/replay formats are versioned and serialized with `serde`.

## Renderer strategy

Start with `NullRenderer` so simulation and tools are testable headlessly. Add a GPU backend behind `RenderBackend`; evaluate `wgpu` first for portability, then direct Vulkan only if profiling proves a need. Target hardware includes Intel HD Graphics 520, so avoid effects that require high-end GPU features.

## Performance budgets

Initial budgets, to be validated by benchmarks:

- 60 FPS target on target hardware.
- 100-student simulation tick: <= 2 ms p95.
- Render submission: <= 4 ms p95 on target GPU.
- Steady-state headless loop: zero heap growth after warmup.
- Cold load to playable test scene: <= 5 s on target SSD.
- Memory: <= 512 MB for vertical slice.

## What not to do

- Do not port `GameObject.Find`, `Camera.main`, or per-frame `GetComponent` patterns.
- Do not put gameplay logic in renderer callbacks.
- Do not use string names as runtime foreign keys.
- Do not copy original C# implementation code or assets into this repository.
