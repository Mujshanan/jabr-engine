# Vendored Code Deviations

This file tracks every Jabr-specific modification to source imported from CivilKit Calcs / CalcpadCE.

## Upstream baseline

- CivilKit repository: https://github.com/civilkit/civilkit-calcs
- Imported revision: a348ab70bc2354386cb1e9f4a80246425f8304f0
- CivilKit version at that revision: 26.11.4
- Imported component: calcpad-core
- Upstream package: civilkit-calcs-engine
- Rust library name: calcpad_core
- Original engine lineage: CalcpadCE, MIT licensed.
- Upstream LICENSE, NOTICE, DEVIATIONS.md, and PORT_LOG.md are preserved under vendor/civilkit-calcs/.

## Import status

**Source import complete.** The complete calcpad-core crate, its tests and examples, and upstream provenance/licensing files are committed under vendor/civilkit-calcs/. The imported source has not been modified by Jabr yet.

The Jabr Math adapter uses public APIs from the vendored crate. Its behavior is implemented outside vendor/ wherever possible. The adapter creates a fresh ExpressionParser for each block evaluation, uses NoFiles, disables data writes, injects synthetic bound assignments, and reads only the outputs declared by Jabr-Math analysis.

## Jabr-specific deviations

### D-001 — Restore Rust 1.75 compatibility

- **Upstream files:** calcpad-core/src/base_types/unit.rs; calcpad-core/src/base_types/real_value.rs (only if required by the compatibility implementation); calcpad-core/src/extension_methods.rs; calcpad-core/src/matrix/matrix_solvers.rs; calcpad-core/src/netfmt.rs; calcpad-core/src/plotter/mod.rs; calcpad-core/src/markdig/inlines.rs; calcpad-core/src/parsers/expression_parser/data_exchange.rs.
- **Upstream behavior:** Uses standard-library APIs newer than the Jabr Kernel-aligned Rust 1.75 baseline, including inline const expressions, iter::repeat_n, Rc::new_uninit, f64::round_ties_even, f64::next_up, and slice::as_chunks.
- **Jabr behavior:** Replace these APIs with Rust 1.75-compatible equivalents. Use std::array::from_fn for Cell arrays, repeat(...).take(...) for repeated characters, a safe Rc::new(Unit::blank(...)) construction, compatibility helpers for ties-to-even rounding and next-up, and chunks_exact(2) for UTF-16 decoding.
- **Reason:** Preserve the previously selected Rust 1.75 MSRV matching Jabr Kernel without changing upstream numerical or parsing semantics.
- **Implementation:** Keep compatibility helpers small and isolated in extension_methods.rs. The Rc construction intentionally trades a small copy for avoiding newer Rc initialization APIs and unsafe initialization.
- **Regression tests:** Ties-to-even cases including negative half-way values and signed zero; next-up behavior for positive/negative finite values, signed zero, infinities and NaN; existing Engine/adapter tests; UTF-16 decoding cases.
- **Upstream conformance impact:** Intended to be behavior-preserving. Re-run CivilKit upstream tests and compare numeric outputs before updating the pinned revision.
- **Rebase/reapply notes:** Recheck whether upstream has adopted compatible implementations; remove or adapt the shim only with tests and a revised deviation entry.
- **Upstream contribution status:** Not proposed.

Do not add entries for adapter code outside vendor/. If a future change edits a vendored file, record it below before considering that change complete.

## Entry template

For every change to vendored source, add an entry containing:
- ID and title.
- Upstream file(s) and symbol(s).
- Upstream behavior.
- Jabr behavior.
- Reason.
- Implementation.
- Regression test(s).
- Upstream conformance impact.
- Rebase/reapply notes.
- Upstream contribution status (not proposed / proposed / accepted / rejected).

Never silently edit vendored source. A source patch without a matching entry and test is incomplete.
