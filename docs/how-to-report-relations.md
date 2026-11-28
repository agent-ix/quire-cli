---
title: Report required relations
type: how-to
description: Report the declared relations that documents must contain.
---

Run `quire relations` to see whether corpus documents contain the relations declared by a module.

## Before you start

Install Quire CLI 0.53.5 and make `quire` available on `PATH`. Run the example from the Quire CLI repository root. Its fixture module declares one required relation.

## Steps

1. **Select one relation.** The report reads documents from the project's `spec/` directory.

   ```sh
   quire relations \
     --scope docs/examples/flow-mode/project \
     --module docs/examples/flow-mode/module \
     --relation connection-has-classification \
     --json
   ```

   The report has one row for `CONN-1`. Its state is `covered`, and the `linked` field identifies `CLS-1`.

2. **Inspect every declared relation.** Omit `--relation` to include all required relations in the loaded modules.

   ```sh
   quire relations \
     --scope docs/examples/flow-mode/project \
     --module docs/examples/flow-mode/module \
     --json
   ```

3. **Fail a gate on incomplete rows.** Add `--strict` to the report command when uncovered, unverified, or partial rows must fail the command.

   ```sh
   quire relations \
     --scope docs/examples/flow-mode/project \
     --module docs/examples/flow-mode/module \
     --strict
   ```

   The full report is written before strict mode exits 1 for an incomplete result.

## Result

Each row reports relation coverage and test verification separately. A row can be `covered` while its linked documents remain `not-tracked` by tests.

## Next steps

- [Inspect a coverage matrix](how-to-inspect-a-coverage-matrix.md)
- [Inspect module-defined flows](how-to-use-relations-flow-mode.md)
