# TODO — KudereSimulator rewrite backlog

This backlog is generated from `tools/generate_plan.py`. Edit the generator or tasks source, then regenerate.
Status values: `todo`, `in-progress`, `blocked`, `review`, `done`.

## Summary

- Total tasks: 130
- Phases: 13
- Priority: P0 = required for vertical slice/foundation, P1 = required for alpha, P2 = parity/polish.
- All code comments and identifiers must be English. Original C# names are preserved in porting metadata, not copied implementation code.

## P0 Foundation

Estimated focused days: 10.5

- [ ] **P0-01 — Confirm redistribution rights and clean-room policy** (`P0`, `producer`, `leads`, 1d)
  - Epic: Governance
  - Description: Record what original code/assets may be referenced, who may access them, and what may be committed.
  - Acceptance: docs/LEGAL.md approved; no original assets in repo; reference-access list exists.
  - Depends on: -

- [ ] **P0-02 — Create reference capture protocol** (`P0`, `qa`, `tools-qa`, 1d)
  - Epic: Governance
  - Description: Define how to record original behavior with video, logs, input scripts, and save-state snapshots without copying source.
  - Acceptance: Reference capture template exists; one sample behavior captured.
  - Depends on: P0-01

- [x] **P0-03 — Set repository, branch, and review rules** (`P0`, `tech-lead`, `leads`, 0.5d)
  - Epic: Governance
  - Description: Initialize repo, protect main, require CI and one engine-area reviewer.
  - Acceptance: main protected; PR template and CODEOWNERS merged.
  - Depends on: P0-01

- [x] **P0-04 — Define performance budgets** (`P0`, `tech-lead`, `engine`, 1d)
  - Epic: Governance
  - Description: Set frame, simulation, render, memory, and load-time budgets for target hardware.
  - Acceptance: Budgets documented and enforced by a benchmark stub.
  - Depends on: P0-01

- [x] **P0-05 — Define deterministic replay format** (`P0`, `engine-dev`, `engine`, 1d)
  - Epic: Governance
  - Description: Specify input stream, RNG seed, tick count, and state hash format.
  - Acceptance: Replay schema documented; two identical runs produce same hash in skeleton.
  - Depends on: P0-04

- [ ] **P0-06 — Create porting map for original scripts** (`P0`, `analyst`, `simulation`, 2d)
  - Epic: Governance
  - Description: Map original C# types to Rust modules/components/systems and mark behavior priority.
  - Acceptance: Top 50 original scripts mapped; owners assigned.
  - Depends on: P0-02

- [ ] **P0-07 — Create asset inventory and licensing sheet** (`P0`, `producer`, `assets`, 2d)
  - Epic: Governance
  - Description: Inventory meshes, textures, audio, animations, fonts, and shaders with replacement/license status.
  - Acceptance: Inventory exists; every asset has license/replacement decision.
  - Depends on: P0-01

- [x] **P0-08 — Set up CI matrix** (`P0`, `tools-dev`, `tools-qa`, 1d)
  - Epic: Governance
  - Description: Run fmt, clippy, tests, and headless runner on Linux and Windows.
  - Acceptance: CI green on skeleton; artifacts include benchmark JSON.
  - Depends on: P0-03

- [ ] **P0-09 — Define issue labels and milestones** (`P1`, `producer`, `leads`, 0.5d)
  - Epic: Governance
  - Description: Create area, priority, size, blocked, and milestone labels.
  - Acceptance: Labels exist; backlog imported from tasks.csv.
  - Depends on: P0-03

- [ ] **P0-10 — Hold vertical-slice planning review** (`P0`, `producer`, `leads`, 0.5d)
  - Epic: Governance
  - Description: Select one school day loop with 10 students, one interaction, and one detection event.
  - Acceptance: Slice scope signed off; tasks assigned; demo date set.
  - Depends on: P0-06

## P1 Core runtime

Estimated focused days: 14

- [x] **P1-01 — Harden fixed-step clock** (`P0`, `engine-dev`, `engine`, 1d)
  - Epic: Core
  - Description: Add pause, time scale, max-delta policy, and deterministic tick counters.
  - Acceptance: Unit tests cover pause, hitch, time scale, and substep budget.
  - Depends on: -

- [ ] **P1-02 — Add engine configuration loading** (`P1`, `engine-dev`, `engine`, 1d)
  - Epic: Core
  - Description: Load TOML/JSON config with validation and defaults.
  - Acceptance: Invalid config produces actionable error; tests cover defaults.
  - Depends on: P1-01

- [ ] **P1-03 — Add structured logging** (`P1`, `engine-dev`, `engine`, 1d)
  - Epic: Core
  - Description: Integrate tracing with frame/tick context and log-level filtering.
  - Acceptance: Logs include tick/frame; no per-frame allocation in hot path.
  - Depends on: P1-01

- [ ] **P1-04 — Add error taxonomy** (`P1`, `engine-dev`, `engine`, 1d)
  - Epic: Core
  - Description: Separate core, asset, render, sim, and IO errors with thiserror.
  - Acceptance: No unwrap in library code except tests; clippy clean.
  - Depends on: P1-03

- [ ] **P1-05 — Add platform abstraction** (`P1`, `engine-dev`, `engine`, 2d)
  - Epic: Core
  - Description: Define time, filesystem, window, and display traits.
  - Acceptance: Headless platform implements traits; desktop platform stub exists.
  - Depends on: P1-02

- [ ] **P1-06 — Add job system foundation** (`P1`, `engine-dev`, `engine`, 3d)
  - Epic: Core
  - Description: Create scoped task graph for parallel system execution with deterministic join points.
  - Acceptance: Benchmarks show speedup; no data races under loom/miri where practical.
  - Depends on: P1-05

- [ ] **P1-07 — Add frame allocator strategy** (`P1`, `engine-dev`, `engine`, 2d)
  - Epic: Core
  - Description: Define per-frame arenas and reset policy for transient data.
  - Acceptance: Frame allocations measured; zero heap growth in steady-state headless loop.
  - Depends on: P1-06

- [x] **P1-08 — Add state hashing** (`P0`, `engine-dev`, `engine`, 1d)
  - Epic: Core
  - Description: Hash simulation state for replay and regression tests.
  - Acceptance: State hash stable across runs; documented exclusions.
  - Depends on: P0-05

- [x] **P1-09 — Add input command buffer** (`P1`, `engine-dev`, `engine`, 1d)
  - Epic: Core
  - Description: Buffer raw input into tick-stamped commands.
  - Acceptance: Replay can feed commands without platform input.
  - Depends on: P1-05

- [ ] **P1-10 — Add engine lifecycle states** (`P1`, `engine-dev`, `engine`, 1d)
  - Epic: Core
  - Description: Implement init, loading, running, paused, shutdown.
  - Acceptance: Lifecycle tests pass; shutdown releases resources.
  - Depends on: P1-05

## P2 ECS and simulation

Estimated focused days: 12.5

- [ ] **P2-01 — Finalize component schema conventions** (`P0`, `tech-lead`, `simulation`, 0.5d)
  - Epic: ECS
  - Description: Define POD components, handles, and forbidden reference types.
  - Acceptance: Schema guide merged; example components reviewed.
  - Depends on: -

- [x] **P2-02 — Add system scheduling stages** (`P0`, `engine-dev`, `simulation`, 1d)
  - Epic: ECS
  - Description: Create explicit stages: input, schedule, AI, movement, perception, events, animation prep.
  - Acceptance: Systems run in declared order; stage graph documented.
  - Depends on: P2-01

- [x] **P2-03 — Add event queues** (`P0`, `engine-dev`, `simulation`, 1d)
  - Epic: ECS
  - Description: Double-buffered typed events with deterministic ordering.
  - Acceptance: Events produced in tick N are consumed in N+1; tests pass.
  - Depends on: P2-02

- [x] **P2-04 — Add stable entity handles** (`P0`, `engine-dev`, `simulation`, 1d)
  - Epic: ECS
  - Description: Generational handles and lookup from original student IDs.
  - Acceptance: Destroyed handle cannot access new entity; ID map tested.
  - Depends on: P2-01

- [x] **P2-05 — Add schedule data model** (`P0`, `sim-dev`, `simulation`, 1d)
  - Epic: ECS
  - Description: Represent daily schedule blocks, destinations, actions, and phase transitions.
  - Acceptance: Schedule serialization round-trips; phase transition tests pass.
  - Depends on: P2-02

- [x] **P2-06 — Add movement system** (`P0`, `sim-dev`, `simulation`, 1d)
  - Epic: ECS
  - Description: Move agents toward destinations with speed, arrival, and pause support.
  - Acceptance: Deterministic movement tests; no per-frame allocations.
  - Depends on: P2-05

- [x] **P2-07 — Add perception system** (`P0`, `sim-dev`, `simulation`, 2d)
  - Epic: ECS
  - Description: Vision range, FOV, occlusion hooks, and attention events.
  - Acceptance: FOV tests pass; occlusion interface documented.
  - Depends on: P2-06

- [x] **P2-08 — Add school clock and phases** (`P0`, `sim-dev`, `simulation`, 1d)
  - Epic: ECS
  - Description: Morning, class, lunch, cleaning, club, end-of-day phases.
  - Acceptance: Clock transitions emit events; schedule system consumes phase.
  - Depends on: P2-05

- [ ] **P2-09 — Add student state machine foundation** (`P0`, `sim-dev`, `simulation`, 2d)
  - Epic: ECS
  - Description: Data-driven states and transitions instead of giant Update branches.
  - Acceptance: State transition table tested; no dynamic dispatch in hot loop.
  - Depends on: P2-08

- [ ] **P2-10 — Add simulation snapshot/restore** (`P1`, `engine-dev`, `simulation`, 2d)
  - Epic: ECS
  - Description: Serialize world state for save/replay and regression.
  - Acceptance: Snapshot restore produces identical state hash.
  - Depends on: P1-08

## P3 Assets and scenes

Estimated focused days: 16

- [x] **P3-01 — Define asset ID and manifest schema** (`P0`, `tools-dev`, `assets`, 1d)
  - Epic: Assets
  - Description: Stable u64 IDs, kind, source path, import settings, and dependencies.
  - Acceptance: Manifest schema documented; duplicate ID rejected.
  - Depends on: -

- [x] **P3-02 — Implement asset registry** (`P0`, `tools-dev`, `assets`, 1d)
  - Epic: Assets
  - Description: Load manifest, resolve IDs, and report missing dependencies.
  - Acceptance: Missing asset error includes ID and source path.
  - Depends on: P3-01

- [ ] **P3-03 — Add glTF mesh importer** (`P1`, `render-dev`, `assets`, 3d)
  - Epic: Assets
  - Description: Import positions, normals, UVs, indices, and skinning metadata.
  - Acceptance: Sample glTF renders in null/renderer test; bounds computed.
  - Depends on: P3-02

- [ ] **P3-04 — Add texture importer** (`P1`, `render-dev`, `assets`, 2d)
  - Epic: Assets
  - Description: Import PNG/JPEG/KTX2 with sRGB and mip settings.
  - Acceptance: Texture metadata round-trips; GPU upload stub tested.
  - Depends on: P3-02

- [ ] **P3-05 — Add material schema** (`P1`, `render-dev`, `assets`, 1d)
  - Epic: Assets
  - Description: PBR material parameters, shader ID, textures, and render queue.
  - Acceptance: Material JSON validates; default material exists.
  - Depends on: P3-04

- [ ] **P3-06 — Add animation clip importer** (`P1`, `anim-dev`, `assets`, 2d)
  - Epic: Assets
  - Description: Import glTF animation tracks and name mapping.
  - Acceptance: Clip duration and track counts tested.
  - Depends on: P3-03

- [ ] **P3-07 — Add audio asset registry** (`P2`, `audio-dev`, `assets`, 1d)
  - Epic: Assets
  - Description: Register audio clips with streaming/decode metadata.
  - Acceptance: Audio manifest loads; missing clip error tested.
  - Depends on: P3-02

- [ ] **P3-08 — Add scene/prefab format** (`P0`, `tools-dev`, `assets`, 2d)
  - Epic: Assets
  - Description: Data-only scene graph with entity templates and component overrides.
  - Acceptance: Scene round-trip tested; prefab override rules documented.
  - Depends on: P3-02

- [ ] **P3-09 — Add asset hot-reload protocol** (`P2`, `tools-dev`, `assets`, 2d)
  - Epic: Assets
  - Description: File-watch manifest and reload safe asset classes.
  - Acceptance: Material/data hot reload works without restart; unsafe reload rejected.
  - Depends on: P3-08

- [ ] **P3-10 — Add replacement asset pipeline** (`P0`, `producer`, `assets`, 1d)
  - Epic: Assets
  - Description: Track original-to-replacement asset mapping and license status.
  - Acceptance: No unlicensed original asset can be packaged by CI check.
  - Depends on: P0-07

## P4 Renderer

Estimated focused days: 24

- [ ] **P4-01 — Finalize render backend trait** (`P0`, `render-dev`, `render`, 1d)
  - Epic: Render
  - Description: Support frames, cameras, meshes, materials, skins, and debug lines.
  - Acceptance: Null renderer implements trait; trait reviewed by render and engine owners.
  - Depends on: -

- [ ] **P4-02 — Select GPU backend** (`P0`, `render-dev`, `render`, 2d)
  - Epic: Render
  - Description: Prototype wgpu vs direct Vulkan for target hardware.
  - Acceptance: Decision record includes Intel HD 520 performance and portability.
  - Depends on: P4-01

- [ ] **P4-03 — Implement device/swapchain init** (`P1`, `render-dev`, `render`, 3d)
  - Epic: Render
  - Description: Create surface, device, queue, swapchain, and resize handling.
  - Acceptance: Headless CI skips GPU; manual smoke test renders clear color.
  - Depends on: P4-02

- [ ] **P4-04 — Implement mesh buffers** (`P1`, `render-dev`, `render`, 2d)
  - Epic: Render
  - Description: Vertex/index buffer upload and draw submission.
  - Acceptance: 10k meshes benchmark meets budget; no per-draw allocation.
  - Depends on: P4-03

- [ ] **P4-05 — Implement camera and culling** (`P1`, `render-dev`, `render`, 2d)
  - Epic: Render
  - Description: Frustum culling and camera matrices from simulation.
  - Acceptance: Culling tests pass; draw count drops in benchmark scene.
  - Depends on: P4-04

- [ ] **P4-06 — Implement material binding** (`P1`, `render-dev`, `render`, 2d)
  - Epic: Render
  - Description: Bind groups, textures, samplers, and material constants.
  - Acceptance: Material switch benchmark documented; validation layers clean.
  - Depends on: P3-05

- [ ] **P4-07 — Implement skeletal rendering** (`P1`, `render-dev`, `render`, 3d)
  - Epic: Render
  - Description: Skinning palette upload and joint matrices.
  - Acceptance: Sample skinned mesh animates correctly.
  - Depends on: P3-06

- [ ] **P4-08 — Implement lighting foundation** (`P2`, `render-dev`, `render`, 3d)
  - Epic: Render
  - Description: Directional light, ambient, and shadow map prototype.
  - Acceptance: Lighting demo scene exists; shadow toggle works.
  - Depends on: P4-06

- [ ] **P4-09 — Implement debug draw** (`P1`, `render-dev`, `render`, 1d)
  - Epic: Render
  - Description: Lines, boxes, spheres, and text overlays for AI/nav/perception.
  - Acceptance: Debug views can be toggled without gameplay changes.
  - Depends on: P4-03

- [ ] **P4-10 — Implement render graph foundation** (`P2`, `render-dev`, `render`, 3d)
  - Epic: Render
  - Description: Explicit passes and resource transitions.
  - Acceptance: Graph validates missing dependencies; frame capture documented.
  - Depends on: P4-08

- [ ] **P4-11 — Add GPU profiling markers** (`P1`, `render-dev`, `render`, 1d)
  - Epic: Render
  - Description: Timestamp queries and pass markers.
  - Acceptance: Benchmark JSON includes GPU pass times where supported.
  - Depends on: P4-10

- [ ] **P4-12 — Add screenshot capture** (`P2`, `tools-dev`, `render`, 1d)
  - Epic: Render
  - Description: Deterministic offscreen capture for visual regression.
  - Acceptance: CI can capture null or GPU reference image.
  - Depends on: P4-03

## P5 Animation

Estimated focused days: 14

- [ ] **P5-01 — Define animation state machine schema** (`P1`, `anim-dev`, `simulation`, 1d)
  - Epic: Animation
  - Description: States, clips, blend trees, transitions, and events.
  - Acceptance: Schema tested; original animation names mapped.
  - Depends on: P3-06

- [ ] **P5-02 — Implement clip sampling** (`P1`, `anim-dev`, `simulation`, 2d)
  - Epic: Animation
  - Description: Sample position/rotation/scale tracks at time.
  - Acceptance: Sampling tests cover loop, clamp, and blend.
  - Depends on: P5-01

- [ ] **P5-03 — Implement blend tree** (`P1`, `anim-dev`, `simulation`, 2d)
  - Epic: Animation
  - Description: 1D/2D blending and additive layers.
  - Acceptance: Blend weights deterministic; no allocation per frame.
  - Depends on: P5-02

- [ ] **P5-04 — Implement animation events** (`P1`, `anim-dev`, `simulation`, 1d)
  - Epic: Animation
  - Description: Tick-stamped events for footsteps, sounds, and gameplay.
  - Acceptance: Events emitted once per crossing; replay stable.
  - Depends on: P5-02

- [ ] **P5-05 — Implement skeleton pose palette** (`P1`, `anim-dev`, `simulation`, 2d)
  - Epic: Animation
  - Description: Compute joint matrices for renderer.
  - Acceptance: Pose matches reference clip within tolerance.
  - Depends on: P5-02

- [ ] **P5-06 — Add locomotion state machine** (`P1`, `anim-dev`, `simulation`, 2d)
  - Epic: Animation
  - Description: Idle, walk, run, turn, and stop states.
  - Acceptance: Student locomotion demo passes visual review.
  - Depends on: P5-03

- [ ] **P5-07 — Add interaction animation hooks** (`P2`, `anim-dev`, `simulation`, 2d)
  - Epic: Animation
  - Description: Mount points and synchronized interaction clips.
  - Acceptance: Two-character interaction sample works.
  - Depends on: P5-04

- [ ] **P5-08 — Add animation compression** (`P2`, `anim-dev`, `assets`, 2d)
  - Epic: Animation
  - Description: Reduce clip memory without visible artifacts.
  - Acceptance: Memory budget met; error tolerance documented.
  - Depends on: P5-05

## P6 Physics and navigation

Estimated focused days: 15

- [ ] **P6-01 — Select physics integration** (`P1`, `engine-dev`, `simulation`, 2d)
  - Epic: Physics
  - Description: Choose rapier3d or custom character controller plus collision queries.
  - Acceptance: Decision record and prototype benchmark approved.
  - Depends on: -

- [ ] **P6-02 — Implement character controller** (`P1`, `sim-dev`, `simulation`, 3d)
  - Epic: Physics
  - Description: Capsule movement, slopes, steps, and collision response.
  - Acceptance: Controller passes slope/stair test scene.
  - Depends on: P6-01

- [ ] **P6-03 — Implement spatial index** (`P1`, `engine-dev`, `simulation`, 2d)
  - Epic: Physics
  - Description: Broadphase for perception, interaction, and triggers.
  - Acceptance: Query benchmark meets budget; deterministic iteration order.
  - Depends on: P6-01

- [ ] **P6-04 — Implement raycast/occlusion queries** (`P1`, `sim-dev`, `simulation`, 1d)
  - Epic: Physics
  - Description: Line-of-sight and interaction raycasts.
  - Acceptance: Perception occlusion tests pass.
  - Depends on: P6-03

- [ ] **P6-05 — Select pathfinding approach** (`P1`, `sim-dev`, `simulation`, 1d)
  - Epic: Navigation
  - Description: Navmesh, grid, or waypoint graph for school layout.
  - Acceptance: Decision record includes authoring workflow.
  - Depends on: P6-03

- [ ] **P6-06 — Implement path queries** (`P1`, `sim-dev`, `simulation`, 3d)
  - Epic: Navigation
  - Description: Find path, smooth path, and replan on blockage.
  - Acceptance: Agents reach destinations in test map.
  - Depends on: P6-05

- [ ] **P6-07 — Implement crowd avoidance** (`P2`, `sim-dev`, `simulation`, 2d)
  - Epic: Navigation
  - Description: Local avoidance and priority rules.
  - Acceptance: 100 agents avoid deadlock in corridor benchmark.
  - Depends on: P6-06

- [ ] **P6-08 — Add navigation debug visualization** (`P2`, `tools-dev`, `tools-qa`, 1d)
  - Epic: Navigation
  - Description: Draw paths, nav regions, and agent targets.
  - Acceptance: Debug overlay toggle works.
  - Depends on: P4-09

## P7 AI and school simulation

Estimated focused days: 22

- [ ] **P7-01 — Implement student daily routine** (`P0`, `sim-dev`, `simulation`, 3d)
  - Epic: AI
  - Description: Spawn, locker, class, lunch, club, and leave behaviors.
  - Acceptance: 10-student day loop runs without manual intervention.
  - Depends on: P2-09

- [ ] **P7-02 — Implement attention and suspicion** (`P1`, `sim-dev`, `simulation`, 2d)
  - Epic: AI
  - Description: Suspicion accumulation, decay, and reporting thresholds.
  - Acceptance: Suspicion tests cover sight, sound, and memory decay.
  - Depends on: P2-07

- [ ] **P7-03 — Implement witness memory** (`P1`, `sim-dev`, `simulation`, 2d)
  - Epic: AI
  - Description: Store witnessed events with confidence and decay.
  - Acceptance: Witness report changes based on elapsed time and evidence.
  - Depends on: P7-02

- [ ] **P7-04 — Implement reputation effects** (`P2`, `sim-dev`, `simulation`, 2d)
  - Epic: AI
  - Description: Reputation changes movement, interaction, and detection behavior.
  - Acceptance: Reputation parameter sweep produces expected behavior bands.
  - Depends on: P7-02

- [ ] **P7-05 — Implement teacher/staff behavior** (`P2`, `sim-dev`, `simulation`, 2d)
  - Epic: AI
  - Description: Class supervision, patrol, and incident response.
  - Acceptance: Teacher responds to incident in test scenario.
  - Depends on: P7-01

- [ ] **P7-06 — Implement rival schedule foundation** (`P2`, `sim-dev`, `simulation`, 2d)
  - Epic: AI
  - Description: Rival-specific schedule blocks and event hooks.
  - Acceptance: One rival day loop runs with event triggers.
  - Depends on: P7-01

- [ ] **P7-07 — Implement dialogue decision hooks** (`P2`, `gameplay-dev`, `gameplay`, 2d)
  - Epic: AI
  - Description: Data-driven dialogue selection based on state and topics.
  - Acceptance: Dialogue state machine emits test conversation.
  - Depends on: P7-04

- [ ] **P7-08 — Implement group behavior** (`P2`, `sim-dev`, `simulation`, 2d)
  - Epic: AI
  - Description: Clubs, conversations, and crowd reactions.
  - Acceptance: Group scenario benchmark remains stable.
  - Depends on: P7-01

- [ ] **P7-09 — Implement incident response** (`P2`, `sim-dev`, `simulation`, 3d)
  - Epic: AI
  - Description: Alarm, panic, flee, report, and lockdown states.
  - Acceptance: Incident scenario produces deterministic event sequence.
  - Depends on: P7-03

- [ ] **P7-10 — Add AI regression scenarios** (`P1`, `qa`, `tools-qa`, 2d)
  - Epic: AI
  - Description: Recorded scenarios with expected event hashes.
  - Acceptance: CI runs 20 scenarios and compares hashes.
  - Depends on: P1-08

## P8 Gameplay

Estimated focused days: 24

- [ ] **P8-01 — Implement player controller** (`P0`, `gameplay-dev`, `gameplay`, 2d)
  - Epic: Gameplay
  - Description: First/third-person movement, camera, and interaction ray.
  - Acceptance: Player can walk, look, and interact in test scene.
  - Depends on: P6-02

- [ ] **P8-02 — Implement interaction system** (`P0`, `gameplay-dev`, `gameplay`, 2d)
  - Epic: Gameplay
  - Description: Prompt, hold, toggle, and carry interactions.
  - Acceptance: Interaction tests cover priority and cancellation.
  - Depends on: P8-01

- [ ] **P8-03 — Implement inventory foundation** (`P1`, `gameplay-dev`, `gameplay`, 2d)
  - Epic: Gameplay
  - Description: Items, equipment, slots, and drop/pickup events.
  - Acceptance: Inventory round-trip and equip tests pass.
  - Depends on: P8-02

- [ ] **P8-04 — Implement item entities** (`P1`, `gameplay-dev`, `gameplay`, 2d)
  - Epic: Gameplay
  - Description: World items with physics, prompts, and ownership.
  - Acceptance: Pickup/drop scenario works deterministically.
  - Depends on: P8-03

- [ ] **P8-05 — Implement sanity/effects foundation** (`P2`, `gameplay-dev`, `gameplay`, 1d)
  - Epic: Gameplay
  - Description: Sanity value and data-driven visual/audio effect hooks.
  - Acceptance: Sanity changes effect parameters without hard-coded branches.
  - Depends on: P8-01

- [ ] **P8-06 — Implement evidence and cleanup** (`P2`, `gameplay-dev`, `gameplay`, 3d)
  - Epic: Gameplay
  - Description: Blood, objects, witness links, and cleanup actions.
  - Acceptance: Evidence state changes alter AI reports in test scenario.
  - Depends on: P7-03

- [ ] **P8-07 — Implement stealth detection** (`P2`, `gameplay-dev`, `gameplay`, 2d)
  - Epic: Gameplay
  - Description: Noise, visibility, disguise, and trespassing hooks.
  - Acceptance: Stealth scenario tests pass.
  - Depends on: P7-02

- [ ] **P8-08 — Implement dialogue UI backend** (`P2`, `gameplay-dev`, `gameplay`, 2d)
  - Epic: Gameplay
  - Description: Topic selection, responses, and state effects.
  - Acceptance: One conversation can be completed headlessly.
  - Depends on: P7-07

- [ ] **P8-09 — Implement mission/objective system** (`P2`, `gameplay-dev`, `gameplay`, 2d)
  - Epic: Gameplay
  - Description: Data-driven objectives, completion, and failure states.
  - Acceptance: Sample mission completes and fails correctly.
  - Depends on: P8-02

- [ ] **P8-10 — Implement save/load game state** (`P1`, `engine-dev`, `gameplay`, 2d)
  - Epic: Gameplay
  - Description: Serialize player, students, clock, inventory, and objectives.
  - Acceptance: Save/load preserves state hash.
  - Depends on: P2-10

- [ ] **P8-11 — Implement settings and accessibility** (`P2`, `gameplay-dev`, `gameplay`, 2d)
  - Epic: Gameplay
  - Description: Graphics, audio, input remapping, subtitles, and color options.
  - Acceptance: Settings persist and apply without restart where safe.
  - Depends on: P1-02

- [ ] **P8-12 — Implement game mode framework** (`P2`, `gameplay-dev`, `gameplay`, 2d)
  - Epic: Gameplay
  - Description: Story, mission, custom, and sandbox mode boundaries.
  - Acceptance: Mode switch loads correct rules and assets.
  - Depends on: P8-09

## P9 UI, input, audio

Estimated focused days: 15

- [ ] **P9-01 — Select UI approach** (`P1`, `ui-dev`, `gameplay`, 1d)
  - Epic: UI
  - Description: Choose egui, custom GPU UI, or middleware for HUD/menus.
  - Acceptance: Decision record and prototype approved.
  - Depends on: -

- [ ] **P9-02 — Implement HUD foundation** (`P1`, `ui-dev`, `gameplay`, 2d)
  - Epic: UI
  - Description: Prompts, subtitles, clock, and status indicators.
  - Acceptance: HUD renders in demo scene and scales with resolution.
  - Depends on: P9-01

- [ ] **P9-03 — Implement menu stack** (`P2`, `ui-dev`, `gameplay`, 2d)
  - Epic: UI
  - Description: Main menu, pause, settings, save/load, and confirmation dialogs.
  - Acceptance: Menu navigation works with keyboard and gamepad.
  - Depends on: P9-02

- [ ] **P9-04 — Implement dialogue UI** (`P2`, `ui-dev`, `gameplay`, 2d)
  - Epic: UI
  - Description: Topic wheel/list, portraits, and response selection.
  - Acceptance: Dialogue UI completes sample conversation.
  - Depends on: P8-08

- [ ] **P9-05 — Implement input mapping** (`P1`, `engine-dev`, `engine`, 2d)
  - Epic: Input
  - Description: Action maps for keyboard, mouse, and gamepad.
  - Acceptance: Remapping persists; replay uses action commands.
  - Depends on: P1-09

- [ ] **P9-06 — Implement gamepad support** (`P2`, `engine-dev`, `engine`, 1d)
  - Epic: Input
  - Description: Hotplug, deadzones, and rumble hooks.
  - Acceptance: Gamepad smoke test passes on target hardware.
  - Depends on: P9-05

- [ ] **P9-07 — Select audio backend** (`P2`, `audio-dev`, `gameplay`, 1d)
  - Epic: Audio
  - Description: Choose kira, rodio, or custom mixer.
  - Acceptance: Decision record includes latency and licensing.
  - Depends on: -

- [ ] **P9-08 — Implement audio emitter system** (`P2`, `audio-dev`, `gameplay`, 2d)
  - Epic: Audio
  - Description: 3D positional audio, listeners, and priority.
  - Acceptance: Audio events from simulation produce mixer commands.
  - Depends on: P9-07

- [ ] **P9-09 — Implement music state machine** (`P2`, `audio-dev`, `gameplay`, 1d)
  - Epic: Audio
  - Description: Calm, suspicion, danger, and event stingers.
  - Acceptance: Music transitions follow AI state in demo.
  - Depends on: P9-08

- [ ] **P9-10 — Implement subtitle and localization hooks** (`P2`, `ui-dev`, `gameplay`, 1d)
  - Epic: Audio
  - Description: String IDs, subtitle timing, and language fallback.
  - Acceptance: Localization table loads; missing key warning tested.
  - Depends on: P9-04

## P10 Tools and editor

Estimated focused days: 17

- [ ] **P10-01 — Implement debug overlay** (`P1`, `tools-dev`, `tools-qa`, 1d)
  - Epic: Tools
  - Description: Frame time, tick, entity count, systems, and memory.
  - Acceptance: Overlay toggles without gameplay impact.
  - Depends on: P1-03

- [ ] **P10-02 — Implement entity inspector** (`P2`, `tools-dev`, `tools-qa`, 2d)
  - Epic: Tools
  - Description: Inspect/edit components and stable IDs at runtime.
  - Acceptance: Inspector can modify student schedule and see effect.
  - Depends on: P10-01

- [ ] **P10-03 — Implement schedule editor** (`P1`, `tools-dev`, `tools-qa`, 3d)
  - Epic: Tools
  - Description: Author schedule blocks and destinations on map.
  - Acceptance: Schedule exports to game format and validates.
  - Depends on: P3-08

- [ ] **P10-04 — Implement scene editor foundation** (`P2`, `tools-dev`, `tools-qa`, 4d)
  - Epic: Tools
  - Description: Place entities, prefabs, lights, and nav markers.
  - Acceptance: Scene saved and loaded by runner.
  - Depends on: P3-08

- [ ] **P10-05 — Implement asset browser** (`P2`, `tools-dev`, `tools-qa`, 2d)
  - Epic: Tools
  - Description: Browse manifest, preview metadata, and find references.
  - Acceptance: Missing dependency report works.
  - Depends on: P3-02

- [ ] **P10-06 — Implement replay recorder/player** (`P1`, `tools-dev`, `tools-qa`, 2d)
  - Epic: Tools
  - Description: Record input commands and play back with state hash.
  - Acceptance: Replay regression CI job passes.
  - Depends on: P0-05

- [x] **P10-07 — Implement benchmark harness** (`P0`, `qa`, `tools-qa`, 2d)
  - Epic: Tools
  - Description: Run standard scenes and emit JSON/CSV metrics.
  - Acceptance: Benchmark compares against budgets and previous run.
  - Depends on: P0-04

- [ ] **P10-08 — Implement crash/diagnostic bundle** (`P2`, `tools-dev`, `tools-qa`, 1d)
  - Epic: Tools
  - Description: Collect logs, config, replay, and system info.
  - Acceptance: Bundle excludes private user data by default.
  - Depends on: P1-03

## P11 Quality and performance

Estimated focused days: 21.5

- [x] **P11-01 — Enforce fmt and clippy in CI** (`P0`, `tools-dev`, `tools-qa`, 0.5d)
  - Epic: Quality
  - Description: Deny warnings for library code and require formatting.
  - Acceptance: CI fails on fmt/clippy violations.
  - Depends on: P0-08

- [ ] **P11-02 — Add unit test coverage target** (`P1`, `qa`, `tools-qa`, 1d)
  - Epic: Quality
  - Description: Set coverage goals for core/sim and publish report.
  - Acceptance: Core/sim coverage reported; critical modules >80%.
  - Depends on: P11-01

- [ ] **P11-03 — Add property tests** (`P2`, `qa`, `tools-qa`, 2d)
  - Epic: Quality
  - Description: Test schedule, movement, and event ordering invariants.
  - Acceptance: Property tests run in CI nightly.
  - Depends on: P2-06

- [ ] **P11-04 — Add fuzz targets** (`P2`, `qa`, `tools-qa`, 2d)
  - Epic: Quality
  - Description: Fuzz asset manifest, replay parser, and schedule loader.
  - Acceptance: Fuzz targets run without crashes for 10 minutes each.
  - Depends on: P3-01

- [ ] **P11-05 — Add memory regression checks** (`P1`, `qa`, `tools-qa`, 1d)
  - Epic: Quality
  - Description: Track steady-state allocations and asset memory.
  - Acceptance: CI flags >5% memory regression.
  - Depends on: P1-07

- [x] **P11-06 — Add frame-time regression checks** (`P0`, `qa`, `tools-qa`, 1d)
  - Epic: Quality
  - Description: Track p50/p95/p99 frame and sim time.
  - Acceptance: CI flags budget violations.
  - Depends on: P10-07

- [ ] **P11-07 — Add visual regression tests** (`P2`, `qa`, `tools-qa`, 2d)
  - Epic: Quality
  - Description: Capture reference frames for key scenes.
  - Acceptance: Visual diff review workflow exists.
  - Depends on: P4-12

- [ ] **P11-08 — Add gameplay regression scenarios** (`P1`, `qa`, `tools-qa`, 2d)
  - Epic: Quality
  - Description: Automate core school day and incident scenarios.
  - Acceptance: Scenario hashes stable across platforms.
  - Depends on: P7-10

- [ ] **P11-09 — Run miri/loom on unsafe/concurrent code** (`P1`, `engine-dev`, `engine`, 2d)
  - Epic: Quality
  - Description: Audit any unsafe blocks and job system synchronization.
  - Acceptance: No undefined behavior or data races in tested modules.
  - Depends on: P1-06

- [ ] **P11-10 — Profile vertical slice on target GPU** (`P1`, `render-dev`, `render`, 2d)
  - Epic: Quality
  - Description: Profile Intel HD 520 and one discrete GPU if available.
  - Acceptance: Profile report identifies top 5 costs and fixes.
  - Depends on: P4-11

- [ ] **P11-11 — Optimize simulation hot paths** (`P1`, `sim-dev`, `simulation`, 3d)
  - Epic: Quality
  - Description: Remove allocations, batch queries, and use SoA where needed.
  - Acceptance: 100-student tick meets budget.
  - Depends on: P11-06

- [ ] **P11-12 — Optimize render submission** (`P1`, `render-dev`, `render`, 3d)
  - Epic: Quality
  - Description: Batch draws, instance meshes, and reduce bind changes.
  - Acceptance: Draw-call budget met in school scene.
  - Depends on: P4-04

## P12 Release and parity

Estimated focused days: 22.5

- [ ] **P12-01 — Package Linux build** (`P1`, `tools-dev`, `tools-qa`, 1d)
  - Epic: Release
  - Description: Produce reproducible Linux binary and asset pack.
  - Acceptance: Clean machine runs demo scene.
  - Depends on: P11-01

- [ ] **P12-02 — Package Windows build** (`P1`, `tools-dev`, `tools-qa`, 2d)
  - Epic: Release
  - Description: Cross-compile or CI-build Windows binary.
  - Acceptance: Windows smoke test passes.
  - Depends on: P12-01

- [ ] **P12-03 — Create asset packaging tool** (`P1`, `tools-dev`, `assets`, 2d)
  - Epic: Release
  - Description: Pack assets with manifest, compression, and checksums.
  - Acceptance: Packed build loads without source assets.
  - Depends on: P3-10

- [ ] **P12-04 — Add versioning and update notes** (`P2`, `producer`, `leads`, 0.5d)
  - Epic: Release
  - Description: Semantic versioning, build ID, and changelog.
  - Acceptance: Release artifact includes version and replay compatibility note.
  - Depends on: P12-01

- [ ] **P12-05 — Complete vertical slice demo** (`P0`, `tech-lead`, `leads`, 3d)
  - Epic: Parity
  - Description: One school day, 10 students, one interaction, one detection event.
  - Acceptance: Demo recorded and compared against reference behavior.
  - Depends on: P7-01

- [ ] **P12-06 — Complete 100-student benchmark** (`P1`, `sim-dev`, `simulation`, 3d)
  - Epic: Parity
  - Description: Full school population with schedules and perception.
  - Acceptance: Benchmark meets frame and memory budgets.
  - Depends on: P11-11

- [ ] **P12-07 — Complete first rival week slice** (`P2`, `gameplay-dev`, `gameplay`, 5d)
  - Epic: Parity
  - Description: Rival schedule, events, and win/lose conditions.
  - Acceptance: Playable week slice reviewed against reference.
  - Depends on: P7-06

- [ ] **P12-08 — Complete asset replacement pass** (`P0`, `producer`, `assets`, 3d)
  - Epic: Parity
  - Description: Replace or license all assets used in slice.
  - Acceptance: Asset audit passes; no original unlicensed assets packaged.
  - Depends on: P3-10

- [ ] **P12-09 — Conduct external playtest** (`P2`, `qa`, `tools-qa`, 2d)
  - Epic: Parity
  - Description: Run structured playtest and collect metrics/feedback.
  - Acceptance: Playtest report with top issues and triage.
  - Depends on: P12-05

- [ ] **P12-10 — Publish internal alpha** (`P2`, `producer`, `leads`, 1d)
  - Epic: Parity
  - Description: Tag alpha, package builds, and document known gaps.
  - Acceptance: Alpha release checklist complete.
  - Depends on: P12-09
