# Student AI

How student behavior works, what is implemented, and what is deliberately not.

## Design

The original `StudentScript` decided everything inside one `Update`: it checked
the timetable, the vision cone, suspicion, and movement in a single body with
early returns. That made behavior order implicit and made any change risky.

Here each responsibility is a separate system over the ECS world, run in a fixed
order once per tick:

| Order | System | Responsibility |
|-------|--------|----------------|
| 1 | `input` | Apply the tick's player command |
| 2 | `schedule` | Select the active timetable block |
| 3 | `perception` | Compute visibility, emit sightings |
| 4 | `report` | Act on suspicion reached last tick |
| 5 | `suspicion` | Integrate suspicion, decay memory |
| 6 | `behavior` | Resolve the state machine, pick escape target |
| 7 | `movement` | Integrate position at the state's speed scale |

Two ordering decisions are load-bearing and covered by tests:

- **`report` runs before `suspicion`.** Otherwise a student that reached the
  report threshold decays below it in the same tick and the report is swallowed.
- **`movement` snaps to the destination before consulting the speed scale.**
  `Attending` has a zero scale, so checking the scale first froze students a few
  centimetres short of their target forever.

## States

`BehaviorState` in `crates/kud_sim/src/behavior.rs`:

| State | Speed scale | Follows timetable | Aware |
|-------|-------------|-------------------|-------|
| `Idle` | 0.0 | yes | yes |
| `Walking` | 1.0 | yes | yes |
| `Attending` | 0.0 | yes | yes |
| `Socializing` | 0.0 | no | yes |
| `Investigating` | 0.6 | no | yes |
| `Fleeing` | 2.0 | no | yes |
| `Reporting` | 1.3 | no | yes |
| `Incapacitated` | 0.0 | no | no |

`next_state` is a pure function of `TransitionInput`, so every transition is
testable without building a world. Suspicion thresholds are ordered
`investigate < flee < report`; the checks run highest first, so de-escalation
drops one band at a time instead of snapping back to the timetable.

An alarm state does not latch. An earlier version returned early for any alarm
state, which meant a student who once fled would flee forever even after
suspicion reached zero. The ordered threshold checks already provide the
hysteresis that motivated that early return.

`Fleeing` computes an escape destination away from the player on the horizontal
plane, `FLEE_DISTANCE` units out. Without this it kept the timetable
destination, so a fleeing student could run toward the threat.

## Suspicion

`Suspicion` holds a `level` in `0.0..=1.0` and per-student `thresholds`.
`advance` integrates using the frame's `dt`, with a visibility factor in
`0.0..=1.0` so a peripheral glimpse builds suspicion more slowly than a direct
look. Visibility fades toward both the cone edge and the range edge, with a
floor of `0.05` so a valid sighting always registers.

Suspicion is a component, not a global, so per-student temperament is a data
change rather than a code change.

## Witness memory

The original witness flags were booleans that never expired: seeing something
once meant remembering it forever at full certainty, so stealth had no time
dimension. `WitnessMemory` stores records with a confidence that decays.

| Kind | Severity | Decay / minute |
|------|----------|----------------|
| `Violence` | 1.0 | 0.002 |
| `Body` | 0.8 | 0.004 |
| `Blood` | 0.6 | 0.01 |
| `SuspiciousItem` | 0.3 | 0.02 |
| `Trespassing` | 0.2 | 0.03 |

Properties that matter:

- **Deduplicated by `(kind, subject)`.** Repeated sightings reinforce one memory
  instead of appending, so a long incident cannot grow the buffer.
- **Bounded.** Default capacity 16, evicting the weakest memory by weight.
- **Decay is idempotent per minute.** `Memory` tracks `decayed_to_minute`
  separately from `last_seen_minute`; sharing one field made advancing to the
  same minute twice double-decay.
- **Absolute time.** Decay keys off `SchoolClock::absolute_minute()`, not
  `minute`, so memories do not appear to travel backwards at midnight.
- **Reporting needs a cause.** High suspicion alone makes a student flee. Staff
  are only involved when there is a witnessed incident, and reporting clears the
  memory and resets suspicion so one incident produces one report.

## Determinism

All of the above is inside the determinism contract in [REPLAY.md](REPLAY.md):

- No wall-clock and no global RNG in any AI system.
- Every iteration is over ECS queries in spawn order or the ordered registry.
- `state_hash` covers the state tag, suspicion level, and memory length and
  weight, so an AI regression fails a golden test rather than passing silently.
- The state tag is an explicit `match`, never `mem::transmute` or
  `std::mem::discriminant`, whose values are not guaranteed across builds.

## Not implemented yet

- **Occlusion.** `visibility` is range and cone only; walls do not block sight.
  P6-04 adds a raycast behind the existing `Perception` boundary.
- **Spatial index.** Perception is O(students) against a single player, which is
  fine at 100. Student-to-student perception needs P6-03 first.
- **Hearing.** No sound propagation yet.
- **Navigation.** Fleeing moves in a straight line and can walk through walls.
  Pathfinding is a separate task; `movement_system` is the integration point.
- **Social groups.** `Socializing` exists as a state but nothing selects it.
- **Staff response.** `IncidentReported` is emitted; no system consumes it yet.

## Performance

100 students, `school_day`, 600 frames, release build, with the full AI stack:

- p50 1 µs, p95 1 µs, p99 2 µs, max 25 µs
- Budget is 2 ms p95 per tick, so there is substantial headroom for occlusion
  and pathfinding.

The first measured frame remains the max; warmup does not fully hide page
faults from the first touch of the world's storage.
