---
title: Check a module before publishing
type: how-to
description: Check whether a module contains restricted clause text before release.
---

Run `quire publish-check` before publishing a module to check its clause-text restrictions.

## Before you start

Install Quire CLI 0.53.5 and make `quire` available on `PATH`. Run the example from the Quire CLI repository root.

## Steps

1. **Check the module.** The example module contains no restricted clause sets.

   ```sh
   quire publish-check \
     --module docs/examples/flow-mode/module \
     --format json
   ```

2. **Read the result.** An empty `restricted` array means the check found no restricted clause sets. A module with restricted clause text exits 2 and names each restricted set.

   ```json
   {"restricted": []}
   ```

## Result

The command exits 0 when the module has no restricted clause sets. Human output says `no restricted clause sets`.

## Next steps

- [Report required relations](how-to-report-relations.md)
