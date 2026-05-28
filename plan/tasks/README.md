# Tasks Index — quire-cli

See `../plan.md` for the full plan, dependency graph, and quality gates.

| ID | Track | Subject | Depends on | Status |
|----|-------|---------|-----------|--------|
| T-001 | A | Cargo.toml + binary scaffold | — | completed |
| T-002 | A | Path-safety module (FR-005) | T-001 | completed |
| T-003 | A | I/O wiring (FR-006, FR-007, FR-008) | T-001 | completed |
| T-004 | A | cargo-deny + unsafe baseline | T-001 | completed |
| T-005 | B | `render` command (FR-001) | T-002, T-003 | completed |
| T-006 | B | `parse` command (FR-002) | T-002, T-003 | completed |
| T-007 | B | `extract` command (FR-003) | T-002, T-003 | completed |
| T-008 | B | `validate` command (FR-004) | T-002, T-003 | completed |
| T-009 | C | Vendor fixtures from quire-rs | T-001 | completed |
| T-010 | C | Happy-path ITs | T-005..008, T-009 | completed |
| T-011 | C | Sandbox ITs | T-002, T-009 | completed |
| T-012 | C | Error-path ITs | T-005..008 | completed |
| T-013 | C | I/O contract ITs | T-005..008 | completed |
| T-014 | C | Static audits (ldd, thin-boundary, deny, unsafe) | T-001 | completed |
| T-015 | C | Network audit (strace) | T-005..008 | completed |
| T-016 | C | Benchmarks (hyperfine p95) | T-005, T-009 | completed |
| T-017 | C | `--help` snapshot | T-005..008 | completed |
| T-018 | D | README + usage docs | T-005..008 | completed |
| T-019 | D | CI workflow updates | T-014, T-015, T-016 | completed |
| T-020 | D | Release tag + publish | G1..G7 | completed |

## Coordination Rules

1. **Track A is sequential**, must land before Track B starts.
2. **Track B subcommands are parallel** with each other but share the path-safety + I/O modules from Track A. Don't fork those modules; if a subcommand needs an extension, land it in Track A first.
3. **Track C ITs are parallel** with each other but each is gated on the subcommand it tests being green in Track B.
4. **Track D is final** — release-time only.
5. **Quality gates G1..G8** must be advanced in order in `plan.md`.
6. Every commit cites the IDs it advances: `Implements T-005, advances FR-001-AC-1`.
