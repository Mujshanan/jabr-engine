# Development Guide

## Status and scope

Jabr Engine is an independent Rust repository. It is developed and tested independently from Jabr-UI, Jabr-Math, Jabr-Kernel, Jabr-Table, and Jabr-Diagram. Do not create a cross-repository Cargo workspace.

The first integration target is the Rust Calcpad calculation core from CivilKit Calcs. It is a backend, not the Jabr worksheet interpreter and not the owner of Jabr variable state.

## Repository layout (target)

    crates/
      jabr-engine-core/
      jabr-engine-adapter-math/
      jabr-engine-adapter-table/       (later)
      jabr-engine-adapter-diagram/     (later)
      jabr-engine-compose/             (later)
      jabr-engine-wasm/                (later)
      jabr-engine-cli/                 (later)
    vendor/
      civilkit-calcs/
        calcpad-core/
    tests/
      fixtures/
      conformance/
    Cargo.toml
    DEVELOPMENT.md
    SPECIFICATIONS.md
    DEVIATIONS.md

Only crates needed for a milestone should be created. The initial implementation starts with Engine Core and the Math adapter contract.

## Upstream source and vendoring

Upstream project: https://github.com/civilkit/civilkit-calcs

- Initial import revision target: a348ab70bc2354386cb1e9f4a80246425f8304f0.
- Source component: calcpad-core (Rust library crate name: calcpad_core).
- Upstream package name: civilkit-calcs-engine.
- Preserve upstream LICENSE, NOTICE, and relevant third-party notices.
- Do not use a Git submodule or runtime Git dependency: the intended result is source vendored inside this repository.
- Do not copy the CivilKit web application, CLI, unrelated export crates, or UI. Start with the calculation core and add more only when a Jabr requirement proves necessary.
- Keep upstream structure recognizable. Avoid broad renames and mechanical refactors.
- Every Jabr-specific source modification must have a matching DEVIATIONS.md entry, rationale, and regression test.

The vendor import is incomplete until the full calcpad-core source tree, required manifests/assets, upstream notices, and relevant tests are present and the vendored crate builds from a local path. The initial scaffold does not claim that import is complete.

## Development milestones

### M0 — Repository foundation
- Document architecture, scope, constraints, and development gates.
- Establish a Rust workspace and first Engine Core crate.
- Define stable block, analysis, value, span, diagnostic, and adapter contracts.
- Keep the public protocol versioned.

### M1 — Calcpad source import
- Import the pinned CivilKit calcpad-core source into vendor/civilkit-calcs/calcpad-core.
- Preserve license and provenance.
- Make the crate build locally without a network dependency.
- Record baseline upstream test status before Jabr patches.

### M2 — Math adapter proof of concept
- Use the Jabr-Math facade for names-defined/read and source spans.
- Materialize resolved inputs as synthetic Calcpad assignments before each block source.
- Evaluate each block in an isolated parser context.
- Extract only declared outputs as structured Jabr values.
- Map diagnostics back to original authored spans.
- Do not make Calcpad's variable map authoritative.

### M3 — Engine execution semantics
- Implement deterministic visual reading order.
- Resolve names against prior block exports and explicit providers.
- Report unresolved references, duplicate definitions, cycles, and failed producers deterministically.
- Preserve sequential worksheet semantics; dependency edges do not silently reorder source execution.

### M4 — External interfaces
- Add Compose, WASM, and CLI after core contracts stabilize.
- Version request/response envelopes.
- Include session/document identity and revision.
- Drop stale UI responses.

### M5 — Incremental evaluation and other domains
- Add dirty-set recalculation only after full recalculation is correct.
- Add Table and Diagram adapters against their public contracts.
- Add cache identity only with session, revision, worksheet digest, Engine version, and adapter/backend versions.

## Build and test

Run:

    cargo fmt --all -- --check
    cargo test --workspace
    cargo clippy --workspace --all-targets -- -D warnings

After vendoring, add explicit checks for native and WASM builds, selected MSRV, no network dependency, upstream conformance tests, Jabr deviation tests, input injection, structured value round trips, source-map stability, isolation, reassignment, and diagnostics.

## Engineering rules

- Adapters analyze a domain block, receive explicit inputs, and return structured outputs and diagnostics.
- Avoid unwrap, expect, and panic in production paths.
- No global mutable parser instance. Calcpad's parser is stateful and not Send/Sync; confine each parser instance to its evaluation context/thread.
- Never return Calcpad HTML/text as the semantic result.
- Never silently coerce an unsupported Jabr value to a string or floating-point approximation.
- Every behavior change to vendored code requires a DEVIATIONS entry and a test.
- Keep commits small and milestone-scoped. Do not merge a pull request without explicit authorization.
