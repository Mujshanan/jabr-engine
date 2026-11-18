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

None yet. Do not add entries for adapter code outside vendor/. If a future change edits a vendored file, record it below before considering that change complete.

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
