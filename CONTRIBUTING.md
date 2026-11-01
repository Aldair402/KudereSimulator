# Contributing

## Setup

```bash
git clone https://github.com/Aldair402/KudereSimulator.git
cd KudereSimulator
rustup component add rustfmt clippy
cargo test --workspace
```

Minimum toolchain: Rust 1.85. The runner is headless; no GPU or display server is
required for the current test suite.

## Before you open a pull request

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If you changed a hot path, also run the benchmark and include before/after numbers:

```bash
cargo run -p kud_runner --release -- --frames=600 --students=100 --hz=60
```

## Pick a task

1. Open [`TODO.md`](TODO.md) or [`tasks/tasks.csv`](tasks/tasks.csv).
2. Choose a `P0` task whose dependencies are `done`.
3. Set `status` to `in-progress` and put your handle in `owner`.
4. If the task ports original behavior, capture the reference behavior first
   (video, input sequence, expected events) — do not copy implementation code.

## Branch and commit conventions

```text
branch:  squad/ID-short-title      e.g. simulation/P2-05-schedule-data
commit:  ID: imperative summary    e.g. P2-05: add schedule block selection
PR:      [ID] Area: summary        e.g. [P2-05] Simulation: schedule data model
```

## Definition of done

- Acceptance criteria in the task are met and demonstrated.
- `fmt`, `clippy -D warnings`, and `cargo test` pass.
- New behavior has tests; ported behavior has a regression scenario.
- No new steady-state heap allocations in the simulation loop without a note.
- Comments and identifiers are English; original names appear only as references.
- [`docs/PORTING_MAP.csv`](docs/PORTING_MAP.csv) updated if original behavior was ported.
- Task status updated in the CSV and backlog regenerated if needed.

## Hard rules

- Never commit decompiled C#, original meshes, textures, audio, or fonts.
- Never commit credentials, tokens, private keys, or machine-specific paths.
- Never introduce non-determinism into `kud_sim` (no wall-clock, no global RNG,
  no iteration over unordered maps where order affects results).
- Keep the dependency direction: `kud_core` must not depend on gameplay or rendering.

## Reporting bugs

Use the bug issue template. Include a replay or the runner command that
reproduces the problem, plus the state hash if you have one.
