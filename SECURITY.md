# Security policy

## Reporting a vulnerability

Do not open a public issue for a security problem. Contact the maintainer
through GitHub (`@Aldair402`) with details and a reproduction.

## Scope

This is a pre-alpha single-player game engine. The most relevant risks are:

- parsing untrusted asset manifests, scenes, or replay files;
- path traversal in the asset import pipeline;
- panics or undefined behavior reachable from malformed data.

## Rules for contributors

- Never commit tokens, private keys, `.env` files, or machine-specific paths.
- Treat all asset and replay input as untrusted: validate lengths and IDs.
- Any `unsafe` block requires a comment explaining the invariant and a test.
