# Assurance export release fixtures

The two historical payloads were emitted by earlier tagged quire-cli releases
(0.32.0 and 0.33.0) against their locked quire-rs engines, using the same
fictional assurance fixture corpus and module. Both carry the earlier
`assurance-v1` shape, with source premise, schema digests and locator digests.

The historical output was regenerated with each tagged CLI binary and compared
byte-for-byte with that release's checked-in `tests/fixtures/assurance/v1.json`.
Both historical releases intentionally produce the same payload bytes for this
fixture because their published schema bytes and fixture inputs are identical.
The current payload is the checked-in golden at
`tests/fixtures/assurance/v1.json`, which is checked against current CLI output
by `it_138_compact_pretty_and_golden_bytes_are_deterministic`.

The current compatibility test treats the payload as data from its named
release, checks its identity-bearing fields, accepts the current payload using
the current engine, and verifies that the current engine refuses each
historical shape before returning a typed export.
