---
id: CONN-1
type: Connection
title: Health-data connection
sourceEnd:
  type: ix://example/flow-mode/PORT-1
targetEnd:
  type: ix://example/flow-mode/PORT-2
relationships:
  - target: CLS-1
    type: classified-as
  - target: BOUNDARY-1
    type: crosses
---
# Health-data connection
