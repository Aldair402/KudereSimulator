#!/usr/bin/env python3
"""Generate TODO.md and tasks/tasks.csv from one backlog source."""
from __future__ import annotations

import csv
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# id, phase, epic, title, description, acceptance, depends, days, role, squad, priority
T = []
def add(phase, epic, title, description, acceptance, depends, days, role, squad, priority="P1"):
    n = sum(1 for x in T if x[1] == phase) + 1
    phase_code = phase.split()[0]
    T.append((f"{phase_code}-{n:02d}", phase, epic, title, description, acceptance, depends, days, role, squad, priority, "todo"))

P = "P0 Foundation"
add(P, "Governance", "Confirm redistribution rights and clean-room policy", "Record what original code/assets may be referenced, who may access them, and what may be committed.", "docs/LEGAL.md approved; no original assets in repo; reference-access list exists.", "-", 1, "producer", "leads", "P0")
add(P, "Governance", "Create reference capture protocol", "Define how to record original behavior with video, logs, input scripts, and save-state snapshots without copying source.", "Reference capture template exists; one sample behavior captured.", "P0-01", 1, "qa", "tools-qa", "P0")
add(P, "Governance", "Set repository, branch, and review rules", "Initialize repo, protect main, require CI and one engine-area reviewer.", "main protected; PR template and CODEOWNERS merged.", "P0-01", 0.5, "tech-lead", "leads", "P0")
add(P, "Governance", "Define performance budgets", "Set frame, simulation, render, memory, and load-time budgets for target hardware.", "Budgets documented and enforced by a benchmark stub.", "P0-01", 1, "tech-lead", "engine", "P0")
add(P, "Governance", "Define deterministic replay format", "Specify input stream, RNG seed, tick count, and state hash format.", "Replay schema documented; two identical runs produce same hash in skeleton.", "P0-04", 1, "engine-dev", "engine", "P0")
add(P, "Governance", "Create porting map for original scripts", "Map original C# types to Rust modules/components/systems and mark behavior priority.", "Top 50 original scripts mapped; owners assigned.", "P0-02", 2, "analyst", "simulation", "P0")
add(P, "Governance", "Create asset inventory and licensing sheet", "Inventory meshes, textures, audio, animations, fonts, and shaders with replacement/license status.", "Inventory exists; every asset has license/replacement decision.", "P0-01", 2, "producer", "assets", "P0")
add(P, "Governance", "Set up CI matrix", "Run fmt, clippy, tests, and headless runner on Linux and Windows.", "CI green on skeleton; artifacts include benchmark JSON.", "P0-03", 1, "tools-dev", "tools-qa", "P0")
add(P, "Governance", "Define issue labels and milestones", "Create area, priority, size, blocked, and milestone labels.", "Labels exist; backlog imported from tasks.csv.", "P0-03", 0.5, "producer", "leads", "P1")
add(P, "Governance", "Hold vertical-slice planning review", "Select one school day loop with 10 students, one interaction, and one detection event.", "Slice scope signed off; tasks assigned; demo date set.", "P0-06", 0.5, "producer", "leads", "P0")

P = "P1 Core runtime"
add(P, "Core", "Harden fixed-step clock", "Add pause, time scale, max-delta policy, and deterministic tick counters.", "Unit tests cover pause, hitch, time scale, and substep budget.", "-", 1, "engine-dev", "engine", "P0")
add(P, "Core", "Add engine configuration loading", "Load TOML/JSON config with validation and defaults.", "Invalid config produces actionable error; tests cover defaults.", "P1-01", 1, "engine-dev", "engine", "P1")
add(P, "Core", "Add structured logging", "Integrate tracing with frame/tick context and log-level filtering.", "Logs include tick/frame; no per-frame allocation in hot path.", "P1-01", 1, "engine-dev", "engine", "P1")
add(P, "Core", "Add error taxonomy", "Separate core, asset, render, sim, and IO errors with thiserror.", "No unwrap in library code except tests; clippy clean.", "P1-03", 1, "engine-dev", "engine", "P1")
add(P, "Core", "Add platform abstraction", "Define time, filesystem, window, and display traits.", "Headless platform implements traits; desktop platform stub exists.", "P1-02", 2, "engine-dev", "engine", "P1")
add(P, "Core", "Add job system foundation", "Create scoped task graph for parallel system execution with deterministic join points.", "Benchmarks show speedup; no data races under loom/miri where practical.", "P1-05", 3, "engine-dev", "engine", "P1")
add(P, "Core", "Add frame allocator strategy", "Define per-frame arenas and reset policy for transient data.", "Frame allocations measured; zero heap growth in steady-state headless loop.", "P1-06", 2, "engine-dev", "engine", "P1")
add(P, "Core", "Add state hashing", "Hash simulation state for replay and regression tests.", "State hash stable across runs; documented exclusions.", "P0-05", 1, "engine-dev", "engine", "P0")
add(P, "Core", "Add input command buffer", "Buffer raw input into tick-stamped commands.", "Replay can feed commands without platform input.", "P1-05", 1, "engine-dev", "engine", "P1")
add(P, "Core", "Add engine lifecycle states", "Implement init, loading, running, paused, shutdown.", "Lifecycle tests pass; shutdown releases resources.", "P1-05", 1, "engine-dev", "engine", "P1")

P = "P2 ECS and simulation"
add(P, "ECS", "Finalize component schema conventions", "Define POD components, handles, and forbidden reference types.", "Schema guide merged; example components reviewed.", "-", 0.5, "tech-lead", "simulation", "P0")
add(P, "ECS", "Add system scheduling stages", "Create explicit stages: input, schedule, AI, movement, perception, events, animation prep.", "Systems run in declared order; stage graph documented.", "P2-01", 1, "engine-dev", "simulation", "P0")
add(P, "ECS", "Add event queues", "Double-buffered typed events with deterministic ordering.", "Events produced in tick N are consumed in N+1; tests pass.", "P2-02", 1, "engine-dev", "simulation", "P0")
add(P, "ECS", "Add stable entity handles", "Generational handles and lookup from original student IDs.", "Destroyed handle cannot access new entity; ID map tested.", "P2-01", 1, "engine-dev", "simulation", "P0")
add(P, "ECS", "Add schedule data model", "Represent daily schedule blocks, destinations, actions, and phase transitions.", "Schedule serialization round-trips; phase transition tests pass.", "P2-02", 1, "sim-dev", "simulation", "P0")
add(P, "ECS", "Add movement system", "Move agents toward destinations with speed, arrival, and pause support.", "Deterministic movement tests; no per-frame allocations.", "P2-05", 1, "sim-dev", "simulation", "P0")
add(P, "ECS", "Add perception system", "Vision range, FOV, occlusion hooks, and attention events.", "FOV tests pass; occlusion interface documented.", "P2-06", 2, "sim-dev", "simulation", "P0")
add(P, "ECS", "Add school clock and phases", "Morning, class, lunch, cleaning, club, end-of-day phases.", "Clock transitions emit events; schedule system consumes phase.", "P2-05", 1, "sim-dev", "simulation", "P0")
add(P, "ECS", "Add student state machine foundation", "Data-driven states and transitions instead of giant Update branches.", "State transition table tested; no dynamic dispatch in hot loop.", "P2-08", 2, "sim-dev", "simulation", "P0")
add(P, "ECS", "Add simulation snapshot/restore", "Serialize world state for save/replay and regression.", "Snapshot restore produces identical state hash.", "P1-08", 2, "engine-dev", "simulation", "P1")

P = "P3 Assets and scenes"
add(P, "Assets", "Define asset ID and manifest schema", "Stable u64 IDs, kind, source path, import settings, and dependencies.", "Manifest schema documented; duplicate ID rejected.", "-", 1, "tools-dev", "assets", "P0")
add(P, "Assets", "Implement asset registry", "Load manifest, resolve IDs, and report missing dependencies.", "Missing asset error includes ID and source path.", "P3-01", 1, "tools-dev", "assets", "P0")
add(P, "Assets", "Add glTF mesh importer", "Import positions, normals, UVs, indices, and skinning metadata.", "Sample glTF renders in null/renderer test; bounds computed.", "P3-02", 3, "render-dev", "assets", "P1")
add(P, "Assets", "Add texture importer", "Import PNG/JPEG/KTX2 with sRGB and mip settings.", "Texture metadata round-trips; GPU upload stub tested.", "P3-02", 2, "render-dev", "assets", "P1")
add(P, "Assets", "Add material schema", "PBR material parameters, shader ID, textures, and render queue.", "Material JSON validates; default material exists.", "P3-04", 1, "render-dev", "assets", "P1")
add(P, "Assets", "Add animation clip importer", "Import glTF animation tracks and name mapping.", "Clip duration and track counts tested.", "P3-03", 2, "anim-dev", "assets", "P1")
add(P, "Assets", "Add audio asset registry", "Register audio clips with streaming/decode metadata.", "Audio manifest loads; missing clip error tested.", "P3-02", 1, "audio-dev", "assets", "P2")
add(P, "Assets", "Add scene/prefab format", "Data-only scene graph with entity templates and component overrides.", "Scene round-trip tested; prefab override rules documented.", "P3-02", 2, "tools-dev", "assets", "P0")
add(P, "Assets", "Add asset hot-reload protocol", "File-watch manifest and reload safe asset classes.", "Material/data hot reload works without restart; unsafe reload rejected.", "P3-08", 2, "tools-dev", "assets", "P2")
add(P, "Assets", "Add replacement asset pipeline", "Track original-to-replacement asset mapping and license status.", "No unlicensed original asset can be packaged by CI check.", "P0-07", 1, "producer", "assets", "P0")

P = "P4 Renderer"
add(P, "Render", "Finalize render backend trait", "Support frames, cameras, meshes, materials, skins, and debug lines.", "Null renderer implements trait; trait reviewed by render and engine owners.", "-", 1, "render-dev", "render", "P0")
add(P, "Render", "Select GPU backend", "Prototype wgpu vs direct Vulkan for target hardware.", "Decision record includes Intel HD 520 performance and portability.", "P4-01", 2, "render-dev", "render", "P0")
add(P, "Render", "Implement device/swapchain init", "Create surface, device, queue, swapchain, and resize handling.", "Headless CI skips GPU; manual smoke test renders clear color.", "P4-02", 3, "render-dev", "render", "P1")
add(P, "Render", "Implement mesh buffers", "Vertex/index buffer upload and draw submission.", "10k meshes benchmark meets budget; no per-draw allocation.", "P4-03", 2, "render-dev", "render", "P1")
add(P, "Render", "Implement camera and culling", "Frustum culling and camera matrices from simulation.", "Culling tests pass; draw count drops in benchmark scene.", "P4-04", 2, "render-dev", "render", "P1")
add(P, "Render", "Implement material binding", "Bind groups, textures, samplers, and material constants.", "Material switch benchmark documented; validation layers clean.", "P3-05", 2, "render-dev", "render", "P1")
add(P, "Render", "Implement skeletal rendering", "Skinning palette upload and joint matrices.", "Sample skinned mesh animates correctly.", "P3-06", 3, "render-dev", "render", "P1")
add(P, "Render", "Implement lighting foundation", "Directional light, ambient, and shadow map prototype.", "Lighting demo scene exists; shadow toggle works.", "P4-06", 3, "render-dev", "render", "P2")
add(P, "Render", "Implement debug draw", "Lines, boxes, spheres, and text overlays for AI/nav/perception.", "Debug views can be toggled without gameplay changes.", "P4-03", 1, "render-dev", "render", "P1")
add(P, "Render", "Implement render graph foundation", "Explicit passes and resource transitions.", "Graph validates missing dependencies; frame capture documented.", "P4-08", 3, "render-dev", "render", "P2")
add(P, "Render", "Add GPU profiling markers", "Timestamp queries and pass markers.", "Benchmark JSON includes GPU pass times where supported.", "P4-10", 1, "render-dev", "render", "P1")
add(P, "Render", "Add screenshot capture", "Deterministic offscreen capture for visual regression.", "CI can capture null or GPU reference image.", "P4-03", 1, "tools-dev", "render", "P2")

P = "P5 Animation"
add(P, "Animation", "Define animation state machine schema", "States, clips, blend trees, transitions, and events.", "Schema tested; original animation names mapped.", "P3-06", 1, "anim-dev", "simulation", "P1")
add(P, "Animation", "Implement clip sampling", "Sample position/rotation/scale tracks at time.", "Sampling tests cover loop, clamp, and blend.", "P5-01", 2, "anim-dev", "simulation", "P1")
add(P, "Animation", "Implement blend tree", "1D/2D blending and additive layers.", "Blend weights deterministic; no allocation per frame.", "P5-02", 2, "anim-dev", "simulation", "P1")
add(P, "Animation", "Implement animation events", "Tick-stamped events for footsteps, sounds, and gameplay.", "Events emitted once per crossing; replay stable.", "P5-02", 1, "anim-dev", "simulation", "P1")
add(P, "Animation", "Implement skeleton pose palette", "Compute joint matrices for renderer.", "Pose matches reference clip within tolerance.", "P5-02", 2, "anim-dev", "simulation", "P1")
add(P, "Animation", "Add locomotion state machine", "Idle, walk, run, turn, and stop states.", "Student locomotion demo passes visual review.", "P5-03", 2, "anim-dev", "simulation", "P1")
add(P, "Animation", "Add interaction animation hooks", "Mount points and synchronized interaction clips.", "Two-character interaction sample works.", "P5-04", 2, "anim-dev", "simulation", "P2")
add(P, "Animation", "Add animation compression", "Reduce clip memory without visible artifacts.", "Memory budget met; error tolerance documented.", "P5-05", 2, "anim-dev", "assets", "P2")

P = "P6 Physics and navigation"
add(P, "Physics", "Select physics integration", "Choose rapier3d or custom character controller plus collision queries.", "Decision record and prototype benchmark approved.", "-", 2, "engine-dev", "simulation", "P1")
add(P, "Physics", "Implement character controller", "Capsule movement, slopes, steps, and collision response.", "Controller passes slope/stair test scene.", "P6-01", 3, "sim-dev", "simulation", "P1")
add(P, "Physics", "Implement spatial index", "Broadphase for perception, interaction, and triggers.", "Query benchmark meets budget; deterministic iteration order.", "P6-01", 2, "engine-dev", "simulation", "P1")
add(P, "Physics", "Implement raycast/occlusion queries", "Line-of-sight and interaction raycasts.", "Perception occlusion tests pass.", "P6-03", 1, "sim-dev", "simulation", "P1")
add(P, "Navigation", "Select pathfinding approach", "Navmesh, grid, or waypoint graph for school layout.", "Decision record includes authoring workflow.", "P6-03", 1, "sim-dev", "simulation", "P1")
add(P, "Navigation", "Implement path queries", "Find path, smooth path, and replan on blockage.", "Agents reach destinations in test map.", "P6-05", 3, "sim-dev", "simulation", "P1")
add(P, "Navigation", "Implement crowd avoidance", "Local avoidance and priority rules.", "100 agents avoid deadlock in corridor benchmark.", "P6-06", 2, "sim-dev", "simulation", "P2")
add(P, "Navigation", "Add navigation debug visualization", "Draw paths, nav regions, and agent targets.", "Debug overlay toggle works.", "P4-09", 1, "tools-dev", "tools-qa", "P2")

P = "P7 AI and school simulation"
add(P, "AI", "Implement student daily routine", "Spawn, locker, class, lunch, club, and leave behaviors.", "10-student day loop runs without manual intervention.", "P2-09", 3, "sim-dev", "simulation", "P0")
add(P, "AI", "Implement attention and suspicion", "Suspicion accumulation, decay, and reporting thresholds.", "Suspicion tests cover sight, sound, and memory decay.", "P2-07", 2, "sim-dev", "simulation", "P1")
add(P, "AI", "Implement witness memory", "Store witnessed events with confidence and decay.", "Witness report changes based on elapsed time and evidence.", "P7-02", 2, "sim-dev", "simulation", "P1")
add(P, "AI", "Implement reputation effects", "Reputation changes movement, interaction, and detection behavior.", "Reputation parameter sweep produces expected behavior bands.", "P7-02", 2, "sim-dev", "simulation", "P2")
add(P, "AI", "Implement teacher/staff behavior", "Class supervision, patrol, and incident response.", "Teacher responds to incident in test scenario.", "P7-01", 2, "sim-dev", "simulation", "P2")
add(P, "AI", "Implement rival schedule foundation", "Rival-specific schedule blocks and event hooks.", "One rival day loop runs with event triggers.", "P7-01", 2, "sim-dev", "simulation", "P2")
add(P, "AI", "Implement dialogue decision hooks", "Data-driven dialogue selection based on state and topics.", "Dialogue state machine emits test conversation.", "P7-04", 2, "gameplay-dev", "gameplay", "P2")
add(P, "AI", "Implement group behavior", "Clubs, conversations, and crowd reactions.", "Group scenario benchmark remains stable.", "P7-01", 2, "sim-dev", "simulation", "P2")
add(P, "AI", "Implement incident response", "Alarm, panic, flee, report, and lockdown states.", "Incident scenario produces deterministic event sequence.", "P7-03", 3, "sim-dev", "simulation", "P2")
add(P, "AI", "Add AI regression scenarios", "Recorded scenarios with expected event hashes.", "CI runs 20 scenarios and compares hashes.", "P1-08", 2, "qa", "tools-qa", "P1")

P = "P8 Gameplay"
add(P, "Gameplay", "Implement player controller", "First/third-person movement, camera, and interaction ray.", "Player can walk, look, and interact in test scene.", "P6-02", 2, "gameplay-dev", "gameplay", "P0")
add(P, "Gameplay", "Implement interaction system", "Prompt, hold, toggle, and carry interactions.", "Interaction tests cover priority and cancellation.", "P8-01", 2, "gameplay-dev", "gameplay", "P0")
add(P, "Gameplay", "Implement inventory foundation", "Items, equipment, slots, and drop/pickup events.", "Inventory round-trip and equip tests pass.", "P8-02", 2, "gameplay-dev", "gameplay", "P1")
add(P, "Gameplay", "Implement item entities", "World items with physics, prompts, and ownership.", "Pickup/drop scenario works deterministically.", "P8-03", 2, "gameplay-dev", "gameplay", "P1")
add(P, "Gameplay", "Implement sanity/effects foundation", "Sanity value and data-driven visual/audio effect hooks.", "Sanity changes effect parameters without hard-coded branches.", "P8-01", 1, "gameplay-dev", "gameplay", "P2")
add(P, "Gameplay", "Implement evidence and cleanup", "Blood, objects, witness links, and cleanup actions.", "Evidence state changes alter AI reports in test scenario.", "P7-03", 3, "gameplay-dev", "gameplay", "P2")
add(P, "Gameplay", "Implement stealth detection", "Noise, visibility, disguise, and trespassing hooks.", "Stealth scenario tests pass.", "P7-02", 2, "gameplay-dev", "gameplay", "P2")
add(P, "Gameplay", "Implement dialogue UI backend", "Topic selection, responses, and state effects.", "One conversation can be completed headlessly.", "P7-07", 2, "gameplay-dev", "gameplay", "P2")
add(P, "Gameplay", "Implement mission/objective system", "Data-driven objectives, completion, and failure states.", "Sample mission completes and fails correctly.", "P8-02", 2, "gameplay-dev", "gameplay", "P2")
add(P, "Gameplay", "Implement save/load game state", "Serialize player, students, clock, inventory, and objectives.", "Save/load preserves state hash.", "P2-10", 2, "engine-dev", "gameplay", "P1")
add(P, "Gameplay", "Implement settings and accessibility", "Graphics, audio, input remapping, subtitles, and color options.", "Settings persist and apply without restart where safe.", "P1-02", 2, "gameplay-dev", "gameplay", "P2")
add(P, "Gameplay", "Implement game mode framework", "Story, mission, custom, and sandbox mode boundaries.", "Mode switch loads correct rules and assets.", "P8-09", 2, "gameplay-dev", "gameplay", "P2")

P = "P9 UI, input, audio"
add(P, "UI", "Select UI approach", "Choose egui, custom GPU UI, or middleware for HUD/menus.", "Decision record and prototype approved.", "-", 1, "ui-dev", "gameplay", "P1")
add(P, "UI", "Implement HUD foundation", "Prompts, subtitles, clock, and status indicators.", "HUD renders in demo scene and scales with resolution.", "P9-01", 2, "ui-dev", "gameplay", "P1")
add(P, "UI", "Implement menu stack", "Main menu, pause, settings, save/load, and confirmation dialogs.", "Menu navigation works with keyboard and gamepad.", "P9-02", 2, "ui-dev", "gameplay", "P2")
add(P, "UI", "Implement dialogue UI", "Topic wheel/list, portraits, and response selection.", "Dialogue UI completes sample conversation.", "P8-08", 2, "ui-dev", "gameplay", "P2")
add(P, "Input", "Implement input mapping", "Action maps for keyboard, mouse, and gamepad.", "Remapping persists; replay uses action commands.", "P1-09", 2, "engine-dev", "engine", "P1")
add(P, "Input", "Implement gamepad support", "Hotplug, deadzones, and rumble hooks.", "Gamepad smoke test passes on target hardware.", "P9-05", 1, "engine-dev", "engine", "P2")
add(P, "Audio", "Select audio backend", "Choose kira, rodio, or custom mixer.", "Decision record includes latency and licensing.", "-", 1, "audio-dev", "gameplay", "P2")
add(P, "Audio", "Implement audio emitter system", "3D positional audio, listeners, and priority.", "Audio events from simulation produce mixer commands.", "P9-07", 2, "audio-dev", "gameplay", "P2")
add(P, "Audio", "Implement music state machine", "Calm, suspicion, danger, and event stingers.", "Music transitions follow AI state in demo.", "P9-08", 1, "audio-dev", "gameplay", "P2")
add(P, "Audio", "Implement subtitle and localization hooks", "String IDs, subtitle timing, and language fallback.", "Localization table loads; missing key warning tested.", "P9-04", 1, "ui-dev", "gameplay", "P2")

P = "P10 Tools and editor"
add(P, "Tools", "Implement debug overlay", "Frame time, tick, entity count, systems, and memory.", "Overlay toggles without gameplay impact.", "P1-03", 1, "tools-dev", "tools-qa", "P1")
add(P, "Tools", "Implement entity inspector", "Inspect/edit components and stable IDs at runtime.", "Inspector can modify student schedule and see effect.", "P10-01", 2, "tools-dev", "tools-qa", "P2")
add(P, "Tools", "Implement schedule editor", "Author schedule blocks and destinations on map.", "Schedule exports to game format and validates.", "P3-08", 3, "tools-dev", "tools-qa", "P1")
add(P, "Tools", "Implement scene editor foundation", "Place entities, prefabs, lights, and nav markers.", "Scene saved and loaded by runner.", "P3-08", 4, "tools-dev", "tools-qa", "P2")
add(P, "Tools", "Implement asset browser", "Browse manifest, preview metadata, and find references.", "Missing dependency report works.", "P3-02", 2, "tools-dev", "tools-qa", "P2")
add(P, "Tools", "Implement replay recorder/player", "Record input commands and play back with state hash.", "Replay regression CI job passes.", "P0-05", 2, "tools-dev", "tools-qa", "P1")
add(P, "Tools", "Implement benchmark harness", "Run standard scenes and emit JSON/CSV metrics.", "Benchmark compares against budgets and previous run.", "P0-04", 2, "qa", "tools-qa", "P0")
add(P, "Tools", "Implement crash/diagnostic bundle", "Collect logs, config, replay, and system info.", "Bundle excludes private user data by default.", "P1-03", 1, "tools-dev", "tools-qa", "P2")

P = "P11 Quality and performance"
add(P, "Quality", "Enforce fmt and clippy in CI", "Deny warnings for library code and require formatting.", "CI fails on fmt/clippy violations.", "P0-08", 0.5, "tools-dev", "tools-qa", "P0")
add(P, "Quality", "Add unit test coverage target", "Set coverage goals for core/sim and publish report.", "Core/sim coverage reported; critical modules >80%.", "P11-01", 1, "qa", "tools-qa", "P1")
add(P, "Quality", "Add property tests", "Test schedule, movement, and event ordering invariants.", "Property tests run in CI nightly.", "P2-06", 2, "qa", "tools-qa", "P2")
add(P, "Quality", "Add fuzz targets", "Fuzz asset manifest, replay parser, and schedule loader.", "Fuzz targets run without crashes for 10 minutes each.", "P3-01", 2, "qa", "tools-qa", "P2")
add(P, "Quality", "Add memory regression checks", "Track steady-state allocations and asset memory.", "CI flags >5% memory regression.", "P1-07", 1, "qa", "tools-qa", "P1")
add(P, "Quality", "Add frame-time regression checks", "Track p50/p95/p99 frame and sim time.", "CI flags budget violations.", "P10-07", 1, "qa", "tools-qa", "P0")
add(P, "Quality", "Add visual regression tests", "Capture reference frames for key scenes.", "Visual diff review workflow exists.", "P4-12", 2, "qa", "tools-qa", "P2")
add(P, "Quality", "Add gameplay regression scenarios", "Automate core school day and incident scenarios.", "Scenario hashes stable across platforms.", "P7-10", 2, "qa", "tools-qa", "P1")
add(P, "Quality", "Run miri/loom on unsafe/concurrent code", "Audit any unsafe blocks and job system synchronization.", "No undefined behavior or data races in tested modules.", "P1-06", 2, "engine-dev", "engine", "P1")
add(P, "Quality", "Profile vertical slice on target GPU", "Profile Intel HD 520 and one discrete GPU if available.", "Profile report identifies top 5 costs and fixes.", "P4-11", 2, "render-dev", "render", "P1")
add(P, "Quality", "Optimize simulation hot paths", "Remove allocations, batch queries, and use SoA where needed.", "100-student tick meets budget.", "P11-06", 3, "sim-dev", "simulation", "P1")
add(P, "Quality", "Optimize render submission", "Batch draws, instance meshes, and reduce bind changes.", "Draw-call budget met in school scene.", "P4-04", 3, "render-dev", "render", "P1")

P = "P12 Release and parity"
add(P, "Release", "Package Linux build", "Produce reproducible Linux binary and asset pack.", "Clean machine runs demo scene.", "P11-01", 1, "tools-dev", "tools-qa", "P1")
add(P, "Release", "Package Windows build", "Cross-compile or CI-build Windows binary.", "Windows smoke test passes.", "P12-01", 2, "tools-dev", "tools-qa", "P1")
add(P, "Release", "Create asset packaging tool", "Pack assets with manifest, compression, and checksums.", "Packed build loads without source assets.", "P3-10", 2, "tools-dev", "assets", "P1")
add(P, "Release", "Add versioning and update notes", "Semantic versioning, build ID, and changelog.", "Release artifact includes version and replay compatibility note.", "P12-01", 0.5, "producer", "leads", "P2")
add(P, "Parity", "Complete vertical slice demo", "One school day, 10 students, one interaction, one detection event.", "Demo recorded and compared against reference behavior.", "P7-01", 3, "tech-lead", "leads", "P0")
add(P, "Parity", "Complete 100-student benchmark", "Full school population with schedules and perception.", "Benchmark meets frame and memory budgets.", "P11-11", 3, "sim-dev", "simulation", "P1")
add(P, "Parity", "Complete first rival week slice", "Rival schedule, events, and win/lose conditions.", "Playable week slice reviewed against reference.", "P7-06", 5, "gameplay-dev", "gameplay", "P2")
add(P, "Parity", "Complete asset replacement pass", "Replace or license all assets used in slice.", "Asset audit passes; no original unlicensed assets packaged.", "P3-10", 3, "producer", "assets", "P0")
add(P, "Parity", "Conduct external playtest", "Run structured playtest and collect metrics/feedback.", "Playtest report with top issues and triage.", "P12-05", 2, "qa", "tools-qa", "P2")
add(P, "Parity", "Publish internal alpha", "Tag alpha, package builds, and document known gaps.", "Alpha release checklist complete.", "P12-09", 1, "producer", "leads", "P2")

PHASES = list(dict.fromkeys(x[1] for x in T))

def md_escape(s: str) -> str:
    return s.replace("|", "\\|")

lines = [
    "# TODO — KudereSimulator rewrite backlog",
    "",
    "This backlog is generated from `tools/generate_plan.py`. Edit the generator or tasks source, then regenerate.",
    "Status values: `todo`, `in-progress`, `blocked`, `review`, `done`.",
    "",
    "## Summary",
    "",
    f"- Total tasks: {len(T)}",
    f"- Phases: {len(PHASES)}",
    "- Priority: P0 = required for vertical slice/foundation, P1 = required for alpha, P2 = parity/polish.",
    "- All code comments and identifiers must be English. Original C# names are preserved in porting metadata, not copied implementation code.",
    "",
]
for phase in PHASES:
    items = [x for x in T if x[1] == phase]
    days = sum(float(x[7]) for x in items)
    lines += [f"## {phase}", "", f"Estimated focused days: {days:g}", ""]
    for tid, ph, epic, title, desc, acc, dep, day, role, squad, pri, status in items:
        lines += [
            f"- [ ] **{tid} — {title}** (`{pri}`, `{role}`, `{squad}`, {day}d)",
            f"  - Epic: {epic}",
            f"  - Description: {desc}",
            f"  - Acceptance: {acc}",
            f"  - Depends on: {dep}",
            "",
        ]
(ROOT / "TODO.md").write_text("\n".join(lines), encoding="utf-8")

(ROOT / "tasks").mkdir(exist_ok=True)
with (ROOT / "tasks/tasks.csv").open("w", newline="", encoding="utf-8") as f:
    w = csv.writer(f)
    w.writerow(["id","phase","epic","title","description","acceptance","depends_on","estimate_days","role","squad","priority","status","owner"])
    for row in T:
        w.writerow(list(row) + [""])

print(f"Generated {len(T)} tasks across {len(PHASES)} phases")
