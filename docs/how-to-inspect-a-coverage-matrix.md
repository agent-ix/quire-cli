---
title: Inspect a coverage matrix
type: how-to
description: Read criterion coverage and design relations for a repository.
---

Run `quire matrix` from a repository root to inspect its computed criterion coverage.

## Before you start

Install Quire CLI 0.53.5 and make `quire` available on `PATH`. Run the commands from the repository root. Have the module paths that define the repository's traceability model.

## Steps

1. **Print the matrix.** Pass the repository root and every module that contributes to its model.

   ```sh
   quire matrix --scope . \
     --module /path/to/spec-artifacts-iso \
     --module /path/to/spec-artifacts-process
   ```

   The Markdown output has one table per requirement. Each criterion row includes its binding status and the source paths that bind it.

2. **Write machine-readable rows.** Use tab-separated values for one record per criterion.

   ```sh
   quire matrix --scope . \
     --module /path/to/spec-artifacts-iso \
     --module /path/to/spec-artifacts-process \
     --format tsv
   ```

   Use `--format json` when a consumer needs the design matrix, parent groups, and engine provenance. Diagnostics go to stderr; the selected report goes to stdout.

3. **Require complete coverage.** Add `--strict` when the matrix is a gate.

   ```sh
   quire matrix --scope . \
     --module /path/to/spec-artifacts-iso \
     --module /path/to/spec-artifacts-process \
     --strict
   ```

   Strict mode exits 1 when a live criterion has an incomplete binding state or a design relation remains uncovered.

## Result

The report lists each criterion, its binding state, and its source paths. JSON adds the design groups and engine provenance.

## Next steps

- [Report required relations](how-to-report-relations.md)
- [Trace a requirement to source](how-to-trace-requirements.md)
