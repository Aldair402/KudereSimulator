# Deterministic replay and state hashing

Task references: P0-05 (replay format), P1-08 (state hashing).

## Why

The original game's behavior depended on MonoBehaviour execution order, wall
clock time, and scene lookup order. None of that is reproducible, so a bug could
not be pinned to a recording. Here a run is a pure function of:

```text
(scenario, sim config, fixed hz, seed, command stream)
```

Same inputs produce the same state hash, on any platform, in debug or release.

## State hash

`Simulation::state_hash()` folds observable state with FNV-1a in a canonical
order:

1. tick count, school minute, school day, student count;
2. for each student **in ascending `StudentId` order**: phase, alive, speed,
   destination, position;
3. player presence and position.

Deliberately excluded:

- ECS entity handles and archetype layout — they depend on spawn order;
- the event queue — it is a tick output, not carried state;
- render and timing data — they must never affect simulation.

Floats are hashed by bit pattern, with `-0.0` normalized to `0.0` so a sign flip
on a zero component is not a false regression.

The hash is **not cryptographic**. It detects accidental divergence.

## Replay format

Version 1, JSON:

```json
{
  "version": 1,
  "fixed_hz": 60,
  "seed": 0,
  "scenario": { "school_day": { "students": 10 } },
  "config": { "minutes_per_tick": 1 },
  "ticks": 600,
  "commands": ["Idle", { "Move": { "x": 127, "z": 0 } }],
  "final_hash": 10825317594988606951
}
```

A replay is **self-contained**: `verify` reads the scenario and config from the
file, not from command-line flags. That makes it impossible to verify a recording
against a different setup by accident.

Input axes are `i8`, quantized to `-127..=127`, so a recording is byte-identical
across platforms and carries no device float noise.

## Commands

```bash
# Record 600 ticks of a school day with 10 students.
cargo run -p kud_runner --release -- record \
  --scenario=school_day --students=10 --ticks=600 --out=run.json

# Verify. Exits non-zero on the first divergent hash.
cargo run -p kud_runner --release -- verify --replay=run.json
```

Failure output names the tick and both hashes:

```text
error: replay desynchronized at tick 600: expected hash 0x963cce80100e71e7, got 0x538021169732d733
```

## Golden fixtures

`crates/kud_sim/tests/regression.rs` pins hashes for fixed scenarios:

| Scenario | Students | Ticks | Hash |
|---|---|---|---|
| `grid` | 100 | 600 | `0x3edad586d9afee41` |
| `school_day` | 10 | 600 | `0x963cce80100e71e7` |
| `school_day` | 100 | 600 | `0xb8fe35e047415c5e` |

CI runs them in **both debug and release**, because an optimization-sensitive
float path would otherwise pass locally and fail in a packaged build.

### When a fixture fails

1. Decide whether the behavior change is intended.
2. If unintended, fix the code — the fixture is correct.
3. If intended, update the constant in the same commit and state what changed
   observably in the commit message.

Never relax the assertion to make a build green.

## Determinism rules for contributors

- No wall-clock reads inside `kud_sim`.
- No global RNG; use the replay seed when randomness is added.
- Never iterate an unordered map where order affects results — use the registry,
  which is a `BTreeMap` keyed by `StudentId`.
- Events are consumed from the previous tick's buffer, so a system cannot react
  to an event emitted later in the same tick.
- Render interpolation alpha must not feed back into simulation.
