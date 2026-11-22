# Assurance export release fixtures

The two historical payloads were emitted by the tagged quire-cli release
binaries against their locked quire-rs engines, using the same fictional
assurance fixture corpus and module. The producer/consumer identities and
captured byte digests are:

| CLI producer | Locked engine | Engine revision | Schema SHA-256 | Payload SHA-256 |
|---|---|---|---|---|
| quire-cli 0.32.0 | quire-rs 0.46.0 | `a874fb641cb70da83c8c8b23f9fea0a44255b88a` | `441e1b8324fe64e234a007216b11867ec937ac4779fcebcdd84b30fa064367e5` | `de0d328711bef6670cf55e5afe025d39b23ec9aedd7bcaad198781833b78c28a` |
| quire-cli 0.33.0 | quire-rs 0.47.1 | `92dbebc49f354f7a3d5050b94cd2d57d9084d99f` | `441e1b8324fe64e234a007216b11867ec937ac4779fcebcdd84b30fa064367e5` | `de0d328711bef6670cf55e5afe025d39b23ec9aedd7bcaad198781833b78c28a` |
| quire-cli 0.52.0 | quire-rs 0.62.2 | `335be94e8490b1d186a4d168eb5b2e7bab9a4f68` | `91866049a3a070ce4fdefaf8dc2ea8df66b4dad375684ee9c21f93b393d28b60` | `b47441a1f9422ed3385e11e41c7218d416111b6aaa119b2ac15980c5aad6e9ed` |

The historical output was regenerated with each tagged CLI binary and compared
byte-for-byte with that release's checked-in `tests/fixtures/assurance/v1.json`.
Both historical releases intentionally produce the same payload bytes for this
fixture because their published schema bytes and fixture inputs are identical.
The current payload digest is the checked-in golden at
`tests/fixtures/assurance/v1.json`, which is checked against current CLI output
by `it_138_compact_pretty_and_golden_bytes_are_deterministic`.

The current compatibility test treats the payload as data from its named
release, checks its identity-bearing fields, accepts the current payload using
the current engine, and verifies that the current engine refuses each
historical shape before returning a typed export.
