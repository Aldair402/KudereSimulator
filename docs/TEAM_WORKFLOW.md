# Team workflow and task distribution

## Squads

| Squad | Owns | Typical roles |
|---|---|---|
| `engine` | core loop, time, jobs, platform, input, memory | tech-lead, engine-dev |
| `simulation` | ECS, schedules, AI, movement, perception, navigation | sim-dev, anim-dev |
| `render` | GPU backend, materials, lighting, animation rendering | render-dev |
| `assets` | importers, manifests, scene/prefab format, licensing | tools-dev, producer |
| `gameplay` | player, interaction, inventory, objectives, UI/audio | gameplay-dev, ui-dev, audio-dev |
| `tools-qa` | CI, benchmarks, replay, editor, regression, packaging | tools-dev, qa |
| `leads` | scope, priorities, cross-squad decisions, release | tech-lead, producer |

## Task source of truth

- `tasks/tasks.csv` is the importable backlog.
- `TODO.md` is the human-readable generated backlog.
- `tools/generate_plan.py` regenerates both.
- `tools/assign_tasks.py` distributes tasks across a team roster.

CSV columns:

```text
id, phase, epic, title, description, acceptance, depends_on,
estimate_days, role, squad, priority, status, owner
```

## Priority

- `P0`: required for foundation or vertical slice.
- `P1`: required for internal alpha.
- `P2`: parity, polish, or stretch.

## Status

```text
todo -> in-progress -> review -> done
              |
              +-> blocked
```

## Assignment algorithm

`tools/assign_tasks.py` uses:

1. Priority: P0, then P1, then P2.
2. Role match first.
3. Squad match second.
4. Least current estimated load third.
5. Optional per-person sprint capacity.

Run:

```bash
python3 tools/assign_tasks.py --team tasks/team.example.csv --output tasks/assignments.csv
```

For sprint planning with capacity:

```bash
python3 tools/assign_tasks.py --team tasks/team.example.csv --capacity 8 --output tasks/sprint-01.csv
```

## Weekly cadence

- Monday: pick P0/P1 tasks, confirm dependencies, set sprint capacity.
- Daily: 10-minute blocker sync; update CSV/status.
- Wednesday: cross-squad integration check.
- Friday: demo, benchmark, regression hash review, retro.

## Definition of Ready

A task is ready when:

- Acceptance criteria are testable.
- Dependencies are done or explicitly stubbed.
- Owner and reviewer are known.
- Original reference behavior is captured if porting gameplay.
- Performance budget is known if touching hot paths.

## Definition of Done

A task is done when:

- Code is English-only and follows `docs/NAMING.md`.
- Unit/integration tests pass.
- `cargo fmt`, `cargo clippy`, and `cargo test` pass.
- No new hot-path allocations without justification.
- Benchmark/regression hash is updated if behavior changed.
- Porting map is updated if original behavior was ported.
- PR has one area reviewer and one cross-squad reviewer for shared interfaces.
- TODO/CSV status is updated.

## Branch and PR rules

- Branch: `squad/ID-short-title`, e.g. `simulation/P2-05-schedule-data`.
- Commit: `ID: imperative summary`, e.g. `P2-05: add schedule block selection`.
- PR title: `[ID] Area: summary`.
- PR must link task ID, acceptance evidence, tests, and benchmark impact.
- No force-push to shared branches after review starts.

## Board columns

```text
Backlog | Ready | In Progress | Blocked | Review | Done
```

## Labels

- Area: `engine`, `simulation`, `render`, `assets`, `gameplay`, `tools-qa`.
- Priority: `P0`, `P1`, `P2`.
- Type: `feature`, `bug`, `perf`, `refactor`, `docs`, `test`, `legal`.
- State: `blocked`, `needs-reference`, `needs-review`, `good-first-task`.

## Conflict resolution

- Interface changes require an ADR in `docs/adr/`.
- Cross-squad disputes go to tech-lead.
- Scope disputes go to producer.
- Legal/asset questions block merging until resolved.
