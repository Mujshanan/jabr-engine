# Jabr Engine Specifications

**Status:** Initial architecture specification. Requirements are normative unless marked provisional or deferred.

## 1. Purpose

Jabr Engine executes authored worksheet blocks and coordinates domain engines. It is the only layer that understands dependencies across math, table, diagram, and other block domains.

It provides deterministic execution, cross-block binding, structured result storage, diagnostics, and a versioned bridge to Jabr-UI.

## 2. Ownership boundaries

### 2.1 Engine owns
- Worksheet execution order and execution lifecycle.
- Mapping stable UI block IDs to Engine/Kernel node identities for a session.
- Cross-block binding resolution.
- Scheduling and recalculation policy.
- Aggregation and lifecycle of transient block results.
- Cross-domain dependency coordination.
- A symbol-to-producer binding table; it stores references to producers, not variable values.
- Versioned request/response protocol and WASM/CLI integration.

### 2.2 Kernel owns
- Generic identity and node IDs.
- Generic graph, ports, edges, and topological/cycle primitives.
- Generic revisions, variable values/change tracking, snapshots/history, spans, diagnostics, and envelopes where defined by the versioned Kernel API.

Engine depends on jabr-kernel-api, never Kernel implementation internals. Kernel must not depend on Math or Engine.

### 2.3 Math owns
- Jabr mathematical syntax and semantics.
- Parsing and AST.
- Expression-level evaluation contract.
- Structured mathematical values and units.
- Per-block names-defined and names-read analysis.
- Stable source spans in authored math source.
- Function/provider semantics.

The Math facade must not expose Calcpad internals as permanent Jabr public API.

### 2.4 Other domains and UI
- Table owns cell/range models and table-specific addressing.
- Diagram owns geometry, diagram document model, and diagram semantics.
- Jabr-UI/BlockSuite/Yjs own authored document state, persistence, collaboration, rendering, and stable authored block IDs.
- Engine results are transient and are not written into the collaborative authored document by Engine.

## 3. Execution semantics

### 3.1 Reading order

The interpreter follows visual reading order:
1. Traverse blocks from top to bottom.
2. Blocks on the same horizontal row are traversed left to right.
3. Row membership and tie-breaking must be deterministic in the document layout model.

Do not use column-major traversal.

### 3.2 Sequential semantics

Worksheet statements execute sequentially in reading order. A definition becomes available according to that execution sequence. A later block cannot retroactively satisfy an earlier read unless an explicit language feature says otherwise.

The dependency graph assists name resolution, cycle detection, invalidation, and diagnostics. It does not independently redefine language execution order.

### 3.3 Block analysis and evaluation

Each domain adapter exposes a narrow contract conceptually equivalent to:
- analyze(block_source) -> BlockAnalysis
- evaluate(block_source, resolved_inputs) -> BlockEvaluation

Analysis returns names defined, names read, stable spans for definitions/references, domain diagnostics, and an explicit failed-analysis state. Failed analysis must not invent exports.

Evaluation returns structured named outputs, structured diagnostics, and optional execution metadata. Only outputs declared by analysis may be exported. Injected inputs and parser-internal variables must not leak into a block's exports.

## 4. Math adapter and Calcpad backend

### 4.1 Backend role

The CivilKit Rust Calcpad core is the first candidate calculation backend. It performs expression parsing/calculation inside the Math adapter. It does not own Jabr worksheet sequencing, cross-block dependency resolution, Jabr variable history, or the public Jabr language.

The backend remains replaceable behind the Jabr-Math contract.

### 4.2 Bound-input injection

For a math block, Engine resolves each read name to a structured input value. The Math adapter converts each input into a valid Calcpad assignment and places synthetic assignments before the authored block source.

Example authored block:

    y = x * 2
    z = y + 3

Bound input: x = 5 m.

Generated backend source:

    x = 5 m
    y = x * 2
    z = y + 3

The generated prefix is adapter-owned and is not authored document content.

### 4.3 Isolation and state authority

- Each block evaluation receives only its declared resolved inputs.
- Parser state must not leak across independent block evaluations.
- Calcpad mutable variable/function/unit state is adapter-local.
- Kernel/Engine remain authoritative for Jabr variable state, revisions, snapshots, and result lifecycle.
- Reassignment and historical-result behavior must be validated before persistent caching is introduced.

### 4.4 Structured values

The adapter returns typed values, not display strings or HTML. The contract must support at minimum scalar numbers and quantities with units; vector and matrix support is required where the Jabr-Math contract and backend can represent them.

Complex values, exact rational/decimal values, uncertainty/approximation tags, and unsupported backend values require explicit compatibility tests and documented conversion rules. Never silently lose precision or dimensional information.

### 4.5 Source mapping

Synthetic assignments must not shift authored source spans. The adapter maintains a source map from generated backend text to original block source. Diagnostics from synthetic prefix lines are classified as binding/input diagnostics; diagnostics from authored source are mapped to stable original spans.

## 5. Identity, sessions, and revisions

- UI supplies stable string block IDs and a document/session identity.
- Engine maintains a session-scoped mapping from stable block ID to Kernel node identity.
- New valid IDs allocate identities; existing IDs reuse them; removed IDs retire; duplicate IDs invalidate the request.
- A reference to a removed/nonexistent block produces a diagnostic rather than silently allocating a producer.
- Every request includes a revision scoped to the document/session.
- Every response echoes the same identity and revision. UI discards stale responses.
- Node IDs remain internal and are not persistent authored identifiers.

## 6. Dependency and failure semantics

- Resolve reads only against definitions available under sequential execution semantics and explicit provider rules.
- A block with failed analysis remains identifiable as a failed producer but exports no guessed names.
- Distinguish nonexistent producer, missing output/port, and failed producer diagnostics.
- Cycles are reported with participating members and affected dependents. Execution must not silently reorder blocks to break a cycle.
- Unrelated blocks should continue when their required inputs are available and their execution is independent of the failure.

## 7. Results and caching

- ResultStore is Engine-owned and transient in the initial implementation.
- Initial implementation performs full recalculation; no cross-request cache is required.
- Results are structured and revision-associated.
- Snapshots/history are deferred to the Kernel/Engine integration contract.
- Incremental dirty-set evaluation is deferred until full recalculation is correct.
- Any future cache key must include session/document identity, revision, worksheet digest, Engine version, and relevant adapter/backend versions.

## 8. External protocol

The canonical protocol is maintained in this repository and is versioned.

Requests include protocol version, request ID, document/session ID, document revision, and ordered blocks with stable string IDs, block kind, authored source, and required layout metadata.

Responses include protocol version, request ID, document/session ID, echoed revision, per-block structured outputs, diagnostics with stable source spans where available, and deterministic ordering.

Kernel envelopes may be used at external boundaries when defined by the versioned Kernel API. Internal adapter calls use typed Rust structures, not serialized envelopes.

## 9. Vendoring and upstream deviations

- CivilKit Calcs is the source of the Rust Calcpad port; CalcpadCE is the upstream mathematical engine lineage.
- The intended import is source vendoring, not a Git submodule or runtime Git dependency.
- Preserve MIT and third-party notices.
- Pin the imported CivilKit revision.
- Every Jabr-specific modification to vendored files must be recorded in DEVIATIONS.md with upstream behavior, Jabr behavior, rationale, source files, and tests.
- Keep upstream files recognizable and minimize patch surface.

## 10. Deferred scope

- Dirty-set incremental recalculation.
- Persistent result cache.
- Snapshot/history policy beyond the Kernel contract.
- Table and Diagram adapters until their public contracts are ready.
- Full WASM/CLI protocol implementation until core semantics stabilize.
- Backend-specific language extensions that would redefine Jabr syntax.
