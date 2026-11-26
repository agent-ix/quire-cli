---
type: how-to
description: Select tests affected by source changes and inspect dependency moves.
---

# Analyze change impact with Quire

Use `quire impact` to see which tests and related implementations may be affected by a local source change. The command reads Git snapshots and trace markers; it does not build or run the analyzed project.

## Before you start

Install Quire CLI and make sure the repository has the requested commits locally. The head snapshot must contain the specification files and traceability module used by the analysis.

Set the repository and revisions you want to compare:

```sh
REPO=/path/to/repository
BASE=$(git -C "$REPO" rev-parse HEAD^)
HEAD=$(git -C "$REPO" rev-parse HEAD)
```

## Steps

1. **Select tests for the source change.**

   ```sh
   quire impact --repo "$REPO" --base "$BASE" --head "$HEAD" --format json
   ```

   The report includes changed symbols, selected test filters, related implementations, uncovered changes, and whether analysis was complete. The test filters are suggestions for your test runner; Quire does not execute them.

2. **Inspect requirement links when needed.**

   ```sh
   quire impact --repo "$REPO" --base "$BASE" --head "$HEAD" \
     --format json --check-tags
   ```

   The report then includes missing, invalid, unavailable, and non-binding link findings. To make any such finding fail the command, add `--fail-on-tag-findings`. That option requires `--check-tags`.

3. **Compare a dependency move.** Pass the local upstream repository and the old and new revisions for each dependency move:

   ```sh
   quire impact --repo "$REPO" --base "$BASE" --head "$HEAD" \
     --format json \
     --dependency-move /path/to/quire-rs "$OLD_ENGINE_REV" "$NEW_ENGINE_REV"
   ```

   Repeat `--dependency-move` to compare more than one dependency. Quire reports each move separately, including changed upstream files and items, local references, selected tests with their reason chains, and uncovered changes. If an upstream repository or revision cannot be read, the report marks that side unavailable instead of treating it as having no impact.

## Expected output

The default JSON report contains `changed_symbols`, `selected_tests`, `related_implementations`, `findings`, and `selection_complete`. With `--dependency-move`, it also contains one `dependency_moves` entry for each requested comparison, including its revisions, upstream changes, local references, selected tests, and uncovered changes.

## Verify the report

Check `selection_complete` before relying on the selected tests. For dependency moves, inspect each entry in `dependency_moves`; an empty `selected_tests` list is meaningful only when the report is resolved and its upstream inputs are available. Use the emitted `filter` values with your project's test runner when you decide to run the suggested tests.
