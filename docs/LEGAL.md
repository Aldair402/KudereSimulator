# Legal and clean-room policy

This project is a rewrite target, not a redistribution of the original game.

## Rules

1. Do not commit original C# source, decompiled source, Unity assets, audio, textures, meshes, animations, fonts, or shaders unless explicit redistribution rights exist.
2. Use the original build only as a behavior reference.
3. Record observable behavior, input sequences, and expected outputs; do not copy implementation code.
4. Preserve original identifiers only as porting metadata, compatibility field names, or documentation references.
5. Write new English comments and new Rust implementation code.
6. Track every replacement asset with license/source metadata.
7. If strict clean-room separation is required, split into:
   - reference team: captures behavior specifications;
   - implementation team: writes Rust from specifications only.

## Asset policy

- Prefer original replacement assets, CC0 assets, or properly licensed assets.
- Keep a license sheet in `assets/LICENSES.csv` once asset production begins.
- CI should eventually reject packaged assets without license metadata.

## Reference access

Only authorized reference analysts should access original game files. Implementation contributors should work from behavior specs, porting maps, and test fixtures unless the project owner explicitly permits direct reference access.
