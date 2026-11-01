# Verification status

Crate names: `kud_*`. Project: KudereSimulator.

Verified on an Artix Linux x86_64 host (Intel HD Graphics 520, headless SSH session).

## Toolchain

- `rustc 1.99.1 (58cf8f9c3 2026-10-13)`
- `cargo 1.99.1 (5f94df478 2026-08-27)`

## Commands run

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p kud_runner --release -- --frames=600 --students=100 --hz=60
```

## Results

- `cargo fmt --all`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed, 11 unit tests.
- Headless runner output:

```json
{"draws":60600,"elapsed_ms":1.490442,"events":100,"fixed_hz":60,"fixed_steps":600,"frames":600,"sim_ticks":600,"students":100}
```

## Scope of verification

This verifies the headless engine skeleton, fixed-step clock, ECS facade, asset registry, null renderer, schedule/movement/perception systems, and runner. It does not verify GPU rendering, windowing, audio, physics, animation, or original-game parity. The verification session was headless, so graphical smoke tests must be run from a desktop session.

## Safety

No original game file was modified. The original `Assembly-CSharp.dll` hash remains unchanged.
