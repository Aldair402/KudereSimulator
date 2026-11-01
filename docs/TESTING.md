# Testing strategy

## Test layers

1. Unit tests: core clock, ECS helpers, asset registry, schedule selection, movement, perception.
2. Integration tests: engine fixed-step loop, headless render submission, runner output.
3. Deterministic replay tests: same input + seed produces same state hash.
4. Property tests: schedule invariants, movement arrival, event ordering.
5. Fuzz tests: manifest parser, replay parser, schedule loader.
6. Benchmark tests: 100/500/1000 students, draw submission, memory growth.
7. Visual regression: offscreen captures once GPU backend exists.
8. Playtest scenarios: school day, detection, incident, cleanup, mission.

## Commands

```bash
cargo test --workspace
cargo run -p kud_runner --release -- --frames=600 --students=100 --hz=60
```

## Regression rule

Any change to simulation behavior must either:

- preserve existing scenario hashes, or
- update the scenario fixture with a reviewed explanation.

## Performance rule

Hot-path changes must include before/after benchmark output. Do not accept allocations in steady-state simulation without a documented reason.
