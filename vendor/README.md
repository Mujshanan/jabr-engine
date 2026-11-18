# Vendored calculation backend

This directory contains the source import of CivilKit Calcs' Rust Calcpad calculation core.

The import is pinned to revision a348ab70bc2354386cb1e9f4a80246425f8304f0 of
https://github.com/civilkit/civilkit-calcs. The import script is scripts/vendor-calcpad.sh.

- calcpad-core/ is upstream source.
- LICENSE and NOTICE preserve upstream copyright and licensing.
- UPSTREAM_DEVIATIONS.md and UPSTREAM_PORT_LOG.md are upstream project records.
- UPSTREAM.md records provenance.
- Jabr-specific changes must be recorded in the root DEVIATIONS.md and covered by tests.

Do not edit upstream files without a corresponding Jabr deviation entry. Prefer a minimal patch
surface so future upstream updates can be compared and reapplied.
