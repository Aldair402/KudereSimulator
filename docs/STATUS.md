# Verification status

Crate names: `kud_*`. Project: KudereSimulator.

Every result below was produced by running the commands shown, not estimated.
CI re-runs the same gates on Ubuntu and Windows for each push.

## Toolchain

- `rustc 1.99.1 (58cf8f9c3 2026-10-13)`
- `cargo 1.99.1 (5f94df478 2026-08-27)`

Earlier batches were verified on the Artix x86_64 host over SSH. The AI batch was
verified locally on x86_64 Linux with the identical toolchain version while the
SSH tunnel was unavailable. Golden hashes match across both machines and across
debug and release builds.

## Commands run

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --release
cargo run -p kud_runner --release -- bench --scenario=school_day --students=100 --frames=600
```

## Results

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed, no warnings.
- `cargo test --workspace`: **114 tests passed**, 0 failed.
- `cargo test --workspace --release`: **114 tests passed**, 0 failed.

### Test breakdown

| Crate / target | Tests |
|----------------|-------|
| `kud_core` | 18 |
| `kud_ecs` | 1 |
| `kud_asset` | 1 |
| `kud_render` | 1 |
| `kud_engine` | 2 |
| `kud_runner` (cli + bench) | 12 |
| `kud_sim` unit | 73 |
| `kud_sim` regression | 6 |

## Benchmarks

Release build, 600 frames, 60 Hz, 30-frame warmup.

| Scenario | Students | p50 | p95 | p99 | max |
|----------|----------|-----|-----|-----|-----|
| `grid` | 100 | 2 µs | 3 µs | 3 µs | 48 µs |
| `school_day` | 100 | 1 µs | 1 µs | 2 µs | 25 µs |

The budget is 2 ms p95 per 100-student tick, so the current AI stack uses well
under one percent of it. The max is consistently the first measured frame;
warmup does not fully hide first-touch page faults.

```json
{"command":"bench","draws":60600,"elapsed_ms":1.147059,"events":300,"fixed_hz":60,"fixed_steps":600,"frame_us":{"count":600,"max":25,"min":1,"p50":1,"p95":1,"p99":2},"frames":600,"scenario":"school_day","sim_ticks":630,"state_hash":"0x73a03d30369dd271","students":100}
```

## Replay round trip

| Check | Expected | Result |
|-------|----------|--------|
| `record` then `verify` | match | match, exit 0 |
| Corrupted `final_hash` | divergence named with tick and both hashes | exit 1 |
| Contradictory CLI flags on `verify` | ignored, file wins | match, exit 0 |
| `--budget-us` below actual p95 | fail with p95 message | exit 1 |

## Determinism

Golden fixtures, identical in debug and release:

| Scenario | Students | Ticks | Hash |
|----------|----------|-------|------|
| `grid` | 100 | 600 | `0x0482d363258121bd` |
| `school_day` | 10 | 600 | `0xc1bd159bb7b3f8e9` |
| `school_day` | 100 | 600 | `0xb4550604a27a975e` |

These changed in the AI batch because `state_hash` now covers behavior state,
suspicion, and witness memory. See [REPLAY.md](REPLAY.md) for the update rule.

## Not verified

- **GPU rendering.** Only `NullRenderer` is exercised. The remote host is
  headless, so a windowed smoke test needs a graphical session.
- **Windows locally.** Covered by CI, not by a local Windows machine.
- **Cold load and memory budgets.** No asset pipeline or real scene yet, so the
  5 s load and 512 MB targets are unmeasured rather than met.
