# Development Guide

## Status and scope

Jabr Engine is an independent Rust repository. It is developed and tested independently from Jabr-UI, Jabr-Math, Jabr-Kernel, Jabr-Table, and Jabr-Diagram. Do not create a cross-repository Cargo workspace.

The first calculation backend is the Rust Calcpad core from CivilKit Calcs. It is a backend, not the Jabr worksheet interpreter and not the owner of Jabr variable state.

## Repository layout

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
    scripts/
      vendor-calcpad.sh
    tests/
      fixtures/
      conformance/

## Upstream source and vendoring

Upstream project: https://github.com/civilkit/civilkit-calcs

- Imported revision: a348ab70bc2354386cb1e9f4a80246425f8304f0.
- Upstream project version at the imported revision: 26.11.4.
- Source component: calcpad-core (Rust library crate name: calcpad_core).
- Upstream package name: civilkit-calcs-engine.
- The full calcpad-core source, tests, examples, LICENSE, NOTICE, upstream DEVIATIONS, and PORT_LOG are committed under vendor/civilkit-calcs/.
- This is a source vendor import, not a Git submodule or runtime Git dependency.
- The vendor import script is scripts/vendor-calcpad.sh. It downloads only the pinned revision and refuses to overwrite an existing vendor tree.
- Do not copy the CivilKit web application, CLI, unrelated export crates, or UI.
- Keep upstream structure recognizable. Avoid broad renames and mechanical refactors.
- Every Jabr-specific source modification must have a matching root DEVIATIONS.md entry, rationale, and regression test.

## Development milestones

### M0 — Repository foundation
- [x] Architecture, scope, and development gates documented.
- [x] Rust workspace and first Engine Core crate.
- [x] Math adapter source-injection and source-map scaffold.
- [x] Pinned Calcpad core source import and license/provenance retention.
- [ ] CI compilation and tests.

### M1 — Calcpad adapter validation
- [x] Add isolated ExpressionParser evaluation context.
- [x] Disable host file reads/writes for block evaluation.
- [x] Inject resolved inputs as synthetic assignments.
- [x] Extract declared outputs as structured values.
- [ ] Validate scalar and quantity round trips against Calcpad.
- [ ] Validate vector/matrix and complex result conversion.
- [ ] Validate parser diagnostics and source-line mapping.
- [ ] Validate native and WASM behavior.

### M2 — Jabr-Math integration
- Connect the adapter's MathAnalysisProvider to the real Jabr-Math public facade.
- Confirm names-defined/read and stable span contracts.
- Confirm function definitions and provider callbacks.
- Preserve exact decimal/rational values in Jabr-Math; tag Calcpad f64 results as approximate.

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

The initial CI gate should compile and test the Engine Core and Math adapter, including the vendored core as a dependency. Run upstream conformance tests separately so their baseline is distinguishable from Jabr adapter tests.

Required adapter tests:
- Input injection with unit-bearing values.
- Structured scalar/vector/matrix/complex result conversion.
- Source-map stability with multiple synthetic assignments.
- Isolation across independent block evaluations.
- Reassignment and result-history behavior.
- Diagnostics mapped to authored spans.
- No filesystem access from worksheet blocks.

## Engineering rules

- Adapters analyze a domain block, receive explicit inputs, and return structured outputs and diagnostics.
- Avoid unwrap, expect, and panic in production paths.
- No global mutable parser instance. Calcpad's parser is stateful and uses Rc/RefCell; confine each parser instance to its evaluation context/thread.
- Native ExpressionParser defaults to local disk access. The Jabr adapter must always install NoFiles and disable data writes.
- Never return Calcpad HTML/text as the semantic result.
- Never silently coerce an unsupported Jabr value to a string or floating-point approximation. Backend approximate values must be tagged.
- Every behavior change to vendored source requires a DEVIATIONS entry and a test.
- Keep commits small and milestone-scoped. Do not merge a pull request without explicit authorization.
