# Engine tests

This directory will hold deterministic worksheet fixtures and end-to-end conformance cases.

Initial test ownership:
- Engine Core unit tests: visual reading order, transient environment, duplicate definitions.
- Math adapter unit tests: safe input formatting, synthetic-source maps, structured Calcpad values, diagnostic mapping.
- Backend conformance tests: compare Jabr adapter results with pinned CivilKit Calcpad behavior.
- Cross-block fixtures: sequential definitions, missing references, failed producers, and later-block updates.

Keep fixture source authored exactly as a worksheet author would enter it. Do not store injected synthetic assignments in authored fixtures.
