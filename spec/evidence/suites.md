---
id: SUITE-004
title: Real-tool integration checks
type: SuiteRegistry
---

| ID | Name | Command | Tool | Evidence Kind |
|---|---|---|---|---|
| SUITE-004 | Real-converter export tests | `make test-export-real` | quire-cli real-tool-test lane | Integration |

`SUITE-004` runs the repository's ignored real-converter export tests. The
lane records each test's result, trace IDs, exact versions of the converter
and support tools, and the source commit used by the run. If a required tool
or the locked build wrapper is unavailable, affected tests are recorded as
`outcome: error` with `config.execution_state: not-executed`; the target exits
nonzero and does not claim a passing live-tool run.
