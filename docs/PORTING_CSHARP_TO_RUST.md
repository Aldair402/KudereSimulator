# Porting C#/Unity to Rust

## Language mapping

| C# | Rust | Notes |
|---|---|---|
| `class` | `struct` + `impl` | Prefer composition over inheritance. |
| `interface` | `trait` | Use traits for backend boundaries, not deep hierarchies. |
| `List<T>` | `Vec<T>` | Preallocate when count is known. |
| `Dictionary<K,V>` | `HashMap<K,V>` | Use stable IDs as keys. |
| `null` | `Option<T>` | Make absence explicit. |
| exceptions | `Result<T, E>` | Use `thiserror` for library errors. |
| `event` | event queue | Double-buffered, deterministic consumption. |
| `foreach` | `for x in iter` | Prefer iterators over index loops when clear. |
| `switch` | `match` | Exhaustive matching for state machines. |
| `[Test]` | `#[test]` | Unit tests live next to code. |
| `Debug.Log` | `tracing` | Structured, level-filtered, no hot-path spam. |

## Unity mapping

| Unity concept | Rust engine replacement |
|---|---|
| `MonoBehaviour` | ECS component + system function |
| `Update()` | fixed-step system stage |
| `LateUpdate()` | post-simulation/animation-prep stage |
| `FixedUpdate()` | fixed simulation tick |
| `GameObject` | ECS entity |
| `Transform` | `Position`, `Rotation`, `Scale` components |
| `GameObject.Find` | stable ID lookup or ECS query |
| `GetComponent` | ECS query or cached handle |
| `Camera.main` | explicit camera entity/resource |
| Coroutine | state machine, timer component, or job |
| ScriptableObject | data asset with stable ID |
| Prefab | entity template / scene prefab data |
| Scene | data-only scene asset |
| Animator | animation state machine + clip sampler |
| NavMeshAgent | navigation path + character controller |
| Physics.Raycast | spatial index raycast query |

## Original identifier policy

Original C# identifiers are preserved as porting references:

```rust
/// Student runtime state. Original reference: `StudentScript`.
pub struct Student {
    /// Original field: `StudentID`.
    pub student_id: u32,
}
```

Use Rust naming in implementation. If save-file or data compatibility requires the original spelling, use `serde` renaming:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub struct StudentSave {
    #[serde(rename = "StudentID")]
    pub student_id: u32,
}
```

## Porting procedure

1. Capture original behavior: video, input sequence, expected events.
2. Identify original type/field names and write them into the porting map.
3. Define Rust components and system boundaries.
4. Write acceptance tests before implementation.
5. Implement the smallest deterministic system.
6. Compare event/state hashes against reference scenarios.
7. Profile and remove allocations from hot paths.
8. Update `TODO.md` task status and porting map.

## High-priority original systems

| Original reference | Rust target | Priority |
|---|---|---|
| `StudentScript` | `kud_sim::Student`, schedule/movement/perception systems | P0 |
| `StudentManagerScript` | student registry, spawn/despawn, stable ID map | P0 |
| `YandereScript` | player controller, interaction, inventory | P0 |
| `ClockScript` | `SchoolClock`, phase events | P0 |
| `ScheduleBlock` | `ScheduleBlock` data asset | P0 |
| `Pathfinder` | navigation module | P1 |
| `DialogueScript` | dialogue state machine + UI backend | P2 |
| `SanityScript` | sanity/effects data-driven system | P2 |
