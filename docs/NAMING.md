# Naming and comment policy

## Hard rules

- All code comments must be English.
- All identifiers must be English.
- Do not copy original comments verbatim; write new comments describing the Rust implementation.
- Preserve original C# identifiers as reference metadata in doc comments, porting maps, and serialized compatibility fields.
- Do not use Spanish identifiers, Spanish comments, or mixed-language names.

## Rust naming

| Item | Convention | Example |
|---|---|---|
| Crate | `snake_case` | `kud_sim` |
| Module | `snake_case` | `schedule_system` |
| Type / trait / enum | `PascalCase` | `Student`, `RenderBackend` |
| Function / method | `snake_case` | `spawn_student` |
| Local variable / field | `snake_case` | `student_id`, `destination` |
| Constant | `SCREAMING_SNAKE_CASE` | `PROJECT_NAME` |
| Lifetime | short lowercase | `'a` |
| Type parameter | `PascalCase` | `T`, `B`, `R` |

## Original-name traceability

Every ported type should include a doc comment:

```rust
/// Original reference: `StudentManagerScript`.
pub struct StudentRegistry;
```

Every ported field should include the original field name when known:

```rust
/// Original field: `CurrentDestination`.
pub destination: Vec3,
```

## Compatibility names

If an external format requires original C# spelling, keep Rust fields idiomatic and rename at the serialization boundary:

```rust
#[serde(rename = "StudentID")]
pub student_id: u32,
```

## Forbidden patterns

- `estudiante`, `contador`, `tmp`, `cosa`, `hacerCosas`.
- Spanish comments such as `// actualizar alumno`.
- Copying large original comment blocks.
- Non-English log messages in library code.
