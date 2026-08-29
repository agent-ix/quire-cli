# Clause-set CLI verification — 2026-08-28

Scope: FR-020, IT-137 through IT-140, and TC-141. All fixtures are original,
synthetic widget data. No restricted source material was used.

## Recorded runs

```text
CARGO_TARGET_DIR=/tmp/quire-cli-clause-target cargo test --test cli_clauses
test result: ok. 4 passed; 0 failed

CARGO_TARGET_DIR=/tmp/quire-cli-clause-target cargo test --bin quire commands::clauses::tests
test result: ok. 2 passed; 0 failed

CARGO_TARGET_DIR=/tmp/quire-cli-clause-target cargo test --test audit_no_network clauses_does_not_open_inet_socket
test result: ok. 1 passed; 0 failed

CARGO_TARGET_DIR=/tmp/quire-cli-clause-target cargo clippy --all-targets -- -D warnings
Finished `dev` profile

CARGO_TARGET_DIR=/tmp/quire-cli-clause-target make test
All unit, integration, contract, static, and no-network tests passed after the
intentional help and provenance snapshots were updated.
```

The first full `make test` run exposed the expected help snapshot addition.
The second exposed the expected `clause_sets` capability addition in the
provenance golden file. Both diffs were reviewed and updated explicitly; the
subsequent complete run passed.

## Result

- IT-137: passed — exact evaluation, three-valued result, provenance, binding schema.
- IT-138: passed — exact diff and diff schema.
- IT-139: passed — deterministic, column-stable TSV.
- IT-140: passed — malformed context, missing exact version, path traversal,
  and digest tampering all fail closed.
- TC-141: passed — TSV structural escaping.
- NFR-004: passed for the new happy path under `strace -f -e trace=network`.
