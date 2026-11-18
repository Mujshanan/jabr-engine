# Vendored Code Deviations

This file tracks every Jabr-specific modification to source imported from CivilKit Calcs / CalcpadCE.

## Upstream baseline

- CivilKit repository: https://github.com/civilkit/civilkit-calcs
- Initial import revision target: a348ab70bc2354386cb1e9f4a80246425f8304f0
- Upstream component: calcpad-core
- Upstream Rust library name: calcpad_core
- Original engine lineage: CalcpadCE, MIT licensed; preserve upstream NOTICE and LICENSE.

## Import status

**Pending source import.** The revision and required notices have been identified, but the complete calcpad-core source tree has not yet been copied into this repository. No Jabr-specific source patch is recorded yet. Do not treat the backend as build-integrated until the vendor import and local build are verified.

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
