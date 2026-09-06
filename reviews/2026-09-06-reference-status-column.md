# Native reference status selection (#409)

## Scope and history

Base: published `bf8f33c340d405993a394c4a34e1bba5e3545503`.
FR-017-AC-21 was specified in `15d8723`; IT-152/153 and the upstream module
fixture were banked in `f4850df`, before changing the engine dependency.
No CLI production logic, schema, severity policy, or discovery behavior changes.

The old engine `cd75ef3089bf283d21b00ee7ccb335bd3ff1470f` fails both new
native tests: module loading rejects the unknown `status_column` field and
produces no JSON. The first published implementing engine is
`b44178df5b39b0ff6a6b0f701150870559ddecf6`. Its real HTTPS Cargo dependency
and QA submodule were fetched; no local source patch participated.

The module fixture is an exact copy of
`agent-ix/qa-corpus@7442f2770880a4ade303fb23d725804bdef454db` at
`modules/variants/reference-status-column-explicit/manifest.yaml`.
The selected upstream object and the fixture both have Git blob identity
`530fb86a7dca36fa02a50fbf3bd0e97ce4c794e8`; the engine's `corpus` gitlink
selects that exact QA revision. Native tests need no ambient QA checkout.

## Controls

- IT-152: two independently selected sections, default `Status` and explicit
  `Coverage Status`, two complete rows and two actual Rust test symbols; both
  default and strict exit 0, equal JSON, 2/2 backed, no findings.
- IT-153: each header is independently replaced with the other table's header.
  The remaining readable table cannot mask the defect. Default exits 0;
  strict exits 1 after emitting equal full JSON. Exactly one missing-column
  diagnostic names the correct declaration, setting, source path, and line.
  Both rows remain backed and have no status lies, isolating the strict cause.
- Existing strict projection/hollow-denominator controls, coverage rendering,
  provenance, and published output schema controls remain unchanged.

During test bring-up the initial test-only JSON assumptions were corrected
against the published `CoverageReport` definition: empty diagnostic/advisory
lists are omitted, and a diagnostic's source field is `path`, not `document`.
No behavioral expectation or source fixture was changed to match output.

## Initial qualification

With engine `b44178df5b39b0ff6a6b0f701150870559ddecf6`, using
`CARGO_TARGET_DIR=/tmp/contract-core-cli409-target`:

```sh
cargo test --locked --offline --test cli_reference_status_column \
  --test cli_coverage_strict --test cli_coverage \
  --test cli_tool_provenance --test output_contract
cargo clippy --locked --offline --all-targets -- -D warnings
```

The five test targets pass (31 tests total), and all-target Clippy passes.
The full `cargo test --locked --offline` suite also passes after aligning the
existing IT-145 exact-pin constant and owning FR-020 compatibility declaration;
historical changelog/log entries retain their prior pins. The original #389
merge identity is now distinguished from the current compatible engine pin.
`cargo fmt --check`, the unsafe-comment audit, and `cargo deny check licenses`
pass (deny reports three unused allowance warnings). Scoped validation of
FR-017/FR-020 exits 0 with ambient duplicate-declaration warnings; this is not
qualification against an exact external declaration stack.

The first sandbox full-suite attempt failed on `PTRACE_TRACEME` restrictions
in the existing no-network audit. The identical suite rerun with tracing
permission passes all nine network-audit tests; no audit was disabled.

This is native reachability of the engine's additive selector and existing
strict policy, not an aggregate assurance verdict, a Quoin verification-lock
promotion, or approval of a broad declaration/template migration.

## Final selected engine

The reviewed engine revision was `11969b707382203fe8df1c92e8a2fb2fa7e0bbbc`,
fetched from the canonical HTTPS repository. That revision was a pre-merge
commit on `contract-agent-core/reference-status-column` and is no longer
reachable: the branch was rebased onto main and squash-merged as
agent-ix/quire-rs#410. The reviewed content landed unchanged at
`616a7e97c0e8c84aedda71dc198e94e3de3d9da6`, and that is the published pin. Relative to the initial `b44178d` qualification, the engine
adds only its exact ISO/process validation-stack pins and associated CI/audit
declarations and quality record; the implementing Rust source and QA gitlink
remain unchanged. Cargo.toml, Cargo.lock, FR-020-CON-1, its IT-145 constant,
and the new changelog/log entries all identify this final pin. Historical
qualification above retains the actual initial revision.

At this final engine pin, the complete native suite passes (223 tests across
32 executable test targets, plus zero doc tests), including the nine real
network-audit controls. All-target Clippy passes. The only Cargo.lock delta
from the original CLI base is the engine source identity. No path or registry
substitute is selected.
