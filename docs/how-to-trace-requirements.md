---
title: Trace a requirement to source
type: how-to
description: Find source claims and citations for a requirement or file.
---

Run `quire trace` to look up requirement claims and citations in a repository's source tree.

## Before you start

Install Quire CLI 0.53.5 and make `quire` available on `PATH`. Run the commands from the repository root. Pass the module that declares its trace model.

## Steps

1. **Look up one exact ID.** The report separates claims from citations.

   ```sh
   quire trace --scope . \
     --module /path/to/traceability-module \
     --id FR-029-AC-1 \
     --json
   ```

   Exact matching is the default. Add `--prefix` to include IDs that share the requested value at an ID boundary.

2. **Inspect one file.** Use an inverse query to list the IDs found in a source file.

   ```sh
   quire trace --scope . \
     --module /path/to/traceability-module \
     --file src/commands/publish_check.rs \
     --format tsv
   ```

3. **Filter noisy citations.** Use `--exclude-path` for paths known to contain generated or fixture citations. The filter drops citations; claims remain in the report.

## Result

The report keeps source claims and citations in separate collections. File queries list matching trace identifiers with their source locations.

## Next steps

- [Inspect a coverage matrix](how-to-inspect-a-coverage-matrix.md)
- [Analyze change impact](how-to-analyze-change-impact.md)
