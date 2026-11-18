# Jabr Engine

Jabr Engine is the worksheet execution and orchestration layer of Jabr Studio.

It coordinates domain adapters, resolves cross-block bindings, evaluates worksheet blocks in deterministic visual reading order, and publishes structured results and diagnostics to the UI.

## Architecture

- **Engine Core** owns worksheet execution, block identity mapping, cross-block binding resolution, scheduling, result lifecycle, and diagnostics aggregation.
- **Math Adapter** analyzes and evaluates math blocks through the Jabr-Math contract. Its first calculation backend is planned to be the vendored Rust Calcpad engine from CivilKit Calcs.
- **Table Adapter** and **Diagram Adapter** connect their respective domain engines in later milestones.
- **Compose** is the integration layer that registers domain adapters and providers.
- **WASM** and **CLI** expose the versioned external request/response protocol.

Jabr Engine does not define mathematical syntax, own table or diagram models, render the document, or persist/collaborate on authored content. It must not replace the Jabr-owned sequential worksheet interpreter with a backend-defined execution model.

## Current status

This repository is at the architecture and initial implementation stage.

- [DEVELOPMENT.md](DEVELOPMENT.md) — milestones, workflow, build and testing.
- [SPECIFICATIONS.md](SPECIFICATIONS.md) — normative architecture and protocol requirements.
- [DEVIATIONS.md](DEVIATIONS.md) — every Jabr-specific modification to vendored upstream code.

The CivilKit Calcpad source is being prepared for an in-repository vendor import. The upstream reference is pinned in DEVIATIONS.md; the source import must preserve upstream notices and licenses. Until the source tree is present and the import is verified, the Calcpad backend is **not yet build-integrated**.

## Execution principles

1. Worksheet blocks execute in document reading order: top-to-bottom, then left-to-right for blocks on the same visual row.
2. Dependency information assists binding and invalidation; it does not silently reorder the authored worksheet or redefine Jabr language semantics.
3. Each math block receives only its resolved bound inputs. The adapter may materialize those inputs as synthetic assignments immediately before the block source.
4. Synthetic source must never shift or corrupt spans reported against authored source.
5. Engine/Kernel state remains authoritative. Calcpad parser state is isolated adapter implementation detail.
6. Results are structured values, not presentation strings or HTML.
7. UI requests and responses carry document/session identity and revision so stale responses can be discarded.

## License

Jabr Engine's original code is licensed under the repository license. Vendored components retain their own upstream licenses and copyright notices; see vendor/ and DEVIATIONS.md.
