# KudereSimulator

A social-stealth simulation game written from scratch in Rust, with its own engine.

This project is a **clean-room rewrite**: no Unity runtime, no decompiled C# in the
repository, and no original game assets. The original game is used only as a
behavior reference, and its identifiers are kept as porting metadata so behavior
can be compared and verified.

> Status: **pre-alpha**. The headless simulation core runs and is tested. There is
> no GPU renderer, window, audio, physics, or playable loop yet. See [`TODO.md`](TODO.md).

## Why Rust

The goal is not "Rust is faster than C#". The goal is architectural: a
deterministic, data-oriented simulation with explicit ownership, no
garbage-collector pauses in the frame loop, and no per-frame scene-graph lookups.
The performance win comes from the design, not from the language alone.

## Architecture

```text
kud_core      fixed-step clock, errors, shared primitives
kud_ecs       ECS facade over hecs
kud_asset     stable asset IDs, manifests, registry
kud_render    render backend trait + headless null renderer
kud_sim       school simulation: schedules, movement, perception
kud_engine    frame loop and subsystem orchestration
kud_runner    headless CLI for tests, benchmarks, and CI
```

Dependency direction is one-way: `kud_core` at the bottom, `kud_runner` at the
top. Gameplay never depends on a GPU API, and the renderer never mutates
simulation state.

Design details: [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) and
[`docs/adr/0001-own-engine.md`](docs/adr/0001-own-engine.md).

## Build and run

Requires Rust 1.85 or newer.

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

# Benchmark with frame-time percentiles.
cargo run -p kud_runner --release -- bench --frames=600 --students=100 --hz=60

# Record and verify a deterministic replay.
cargo run -p kud_runner --release -- record --scenario=school_day --students=10 --ticks=600 --out=run.json
cargo run -p kud_runner --release -- verify --replay=run.json
```

The runner is headless, so it works over SSH and in CI without a display server.
Example output:

```json
{"command":"bench","scenario":"school_day","frames":600,"students":100,"fixed_hz":60,
 "fixed_steps":600,"draws":60600,"events":300,"frame_us":{"p50":2,"p95":2,"p99":4,"max":15},
 "state_hash":"0xeb8f9671cfa68e1d"}
```

Determinism is enforced by golden state hashes checked in debug **and** release.
See [`docs/REPLAY.md`](docs/REPLAY.md).

## Contributing

The backlog is machine-readable and designed for a team:

- [`TODO.md`](TODO.md) — 130 tasks across 13 phases, with acceptance criteria.
- [`tasks/tasks.csv`](tasks/tasks.csv) — importable backlog with owners and estimates.
- [`docs/TEAM_WORKFLOW.md`](docs/TEAM_WORKFLOW.md) — squads, review rules, definition of done.
- [`docs/REPLAY.md`](docs/REPLAY.md) — replay format, state hashing, golden fixtures.
- [`docs/AI.md`](docs/AI.md) — student states, suspicion, witness memory, and known gaps.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — setup, branch naming, and PR checklist.

Distribute work across a roster:

```bash
python3 tools/assign_tasks.py --team tasks/team.example.csv --output tasks/assignments.csv
python3 tools/assign_tasks.py --team tasks/team.example.csv --capacity 8 --output tasks/sprint-01.csv
```

Regenerate the backlog after editing the plan source:

```bash
python3 tools/generate_plan.py
```

## Project rules

- All code comments and identifiers are **English**.
- Original C# names are preserved as doc-comment references and, where a file
  format demands it, via `#[serde(rename = "...")]`.
- No `GameObject.Find`, `Camera.main`, or per-frame `GetComponent` patterns.
- Simulation is deterministic: fixed step, quantized input, double-buffered events,
  canonical state hashing verified against golden fixtures.
- No original game code or assets are committed. See [`docs/LEGAL.md`](docs/LEGAL.md).

## License

Licensed under the GNU General Public License v3.0 or later
([`LICENSE`](LICENSE)). SPDX: `GPL-3.0-or-later`.

This applies only to the original code in this repository. It grants no rights
to any third-party game, asset, or trademark.
