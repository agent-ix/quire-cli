# =============================================================================
# quire-cli Makefile
# =============================================================================

CARGO ?= cargo

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make test-export-real - Export tests that need real pandoc, typst, unzip, pdftotext"
	@echo "  make docs             - strict workspace rustdoc"
	@echo "  make spec             - Validate changed assurance specs and traceability"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - cargo deny check (advisories, bans, licenses, sources)"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make dist-test        - Test the release tooling (verify-binary)"
	@echo "  make bench            - Latency budget (NFR-001): p95 of a quire invocation ≤ 50 ms (needs hyperfine)"
	@echo "  make ci               - All CI gates locally, including specification traceability"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: lint
lint:
	$(CARGO) clippy --workspace --locked --all-targets --all-features -- -D warnings

.PHONY: test
test:
	$(CARGO) test --workspace --locked --all-targets --all-features

# Real-converter export tests (FR-031-AC-1, FR-032-AC-7). Needs pandoc, typst,
# unzip and pdftotext on PATH; the test fails naming the first missing tool.
.PHONY: test-export-real
test-export-real:
	$(CARGO) test --locked --test cli_export -- --ignored

.PHONY: docs
docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --workspace --locked --no-deps --all-features

.PHONY: build
build:
	$(CARGO) build --locked --release

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny --locked check

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

.PHONY: dist-test
dist-test:
	$(CARGO) test --locked -p quire-dist

.PHONY: audit-unsafe
audit-unsafe:
	$(CARGO) run --locked -p quire-qualify -- unsafe-comments

# =============================================================================
# Specification and traceability
# =============================================================================

ASSURANCE_SPEC_DOCS := \
	spec/functional/FR-020-assurance-export-subcommand.md \
	spec/stakeholder/StR-004-thin-boundary-over-quire-rs.md \
	spec/usecase/US-006-export-assurance-facts.md \
	reviews/SR-009-assurance-export-spec-review.md \
	reviews/SR-010-assurance-export-code-review.md \
	reviews/SR-011-assurance-export-gap-analysis.md \
	reviews/SR-012-assurance-export-independent-code-review.md \
	reviews/SR-013-assurance-export-independent-gap-analysis.md \
	reviews/SR-014-assurance-export-review-remediation.md \
	reviews/SR-015-assurance-export-remediation-gap-analysis.md

.PHONY: spec
spec:
	$(CARGO) run --locked -- validate --scope . $(ASSURANCE_SPEC_DOCS) --summary
	$(CARGO) run --locked -- coverage --scope . --json > /tmp/quire-cli-assurance-coverage.json
	$(CARGO) run --locked -p quire-qualify -- assurance-traceability /tmp/quire-cli-assurance-coverage.json

# =============================================================================
# Composite
# =============================================================================

.PHONY: audit-thin-boundary
audit-thin-boundary:
	$(CARGO) run --locked -p quire-qualify -- thin-boundary

.PHONY: deny-bans
deny-bans:
	$(CARGO) deny --locked check bans

# =============================================================================
# Latency budget (NFR-001)
# =============================================================================
# p95 wall-clock of a representative `quire` invocation (validate a conformant
# ISO doc against the bundled module) must stay within the 50 ms budget. Uses
# hyperfine to measure the real release binary end-to-end (process spawn + module
# load + parse + validate), then gates on the computed p95.

BENCH_P95_MS ?= 50

.PHONY: bench
bench:
	$(CARGO) build --locked --release
	hyperfine --shell=none --warmup 5 --runs 50 --export-json /tmp/quire-cli-bench.json \
		'$(CURDIR)/target/release/quire validate $(CURDIR)/tests/fixtures/validate-mod/docs/valid-fr.md --module $(CURDIR)/tests/fixtures/validate-mod'
	$(CARGO) run --locked -p quire-qualify -- bench-p95 /tmp/quire-cli-bench.json $(BENCH_P95_MS)

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test docs dist-test deny cargo-audit audit-unsafe audit-thin-boundary spec
