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
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - cargo deny check licenses"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make bench            - Latency budget (NFR-001): p95 of a quire invocation ≤ 50 ms (needs hyperfine)"
	@echo "  make ci               - All CI gates locally (fmt-check + lint + test + deny + audit-unsafe)"

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
	$(CARGO) clippy --all-targets -- -D warnings

.PHONY: test
test:
	$(CARGO) test

.PHONY: build
build:
	$(CARGO) build --release

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny check licenses

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

# =============================================================================
# Composite
# =============================================================================

.PHONY: audit-thin-boundary
audit-thin-boundary:
	bash scripts/check_thin_boundary.sh

.PHONY: deny-bans
deny-bans:
	$(CARGO) deny check bans

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
	$(CARGO) build --release
	hyperfine --shell=none --warmup 5 --runs 50 --export-json /tmp/quire-cli-bench.json \
		'$(CURDIR)/target/release/quire validate $(CURDIR)/tests/fixtures/iso-docs/FR-valid.md --module $(CURDIR)/tests/fixtures/iso'
	@python3 -c "import json; \
r=json.load(open('/tmp/quire-cli-bench.json'))['results'][0]['times']; \
r.sort(); \
p95=r[max(0,int(len(r)*0.95)-1)]*1000.0; \
print(f'p95={p95:.2f}ms (budget {$(BENCH_P95_MS)}ms, n={len(r)})'); \
exit(0 if p95 <= $(BENCH_P95_MS) else 1)"

# =============================================================================
# Fixtures
# =============================================================================

QUIRE_RS_ISO ?= ../quire-rs/tests/fixtures/modules/iso

.PHONY: refresh-fixtures
refresh-fixtures:
	@if [ ! -d "$(QUIRE_RS_ISO)" ]; then \
		echo "upstream ISO fixtures not found at $(QUIRE_RS_ISO); set QUIRE_RS_ISO=" >&2; exit 1; \
	fi
	rm -rf tests/fixtures/iso
	mkdir -p tests/fixtures/iso
	cp -r $(QUIRE_RS_ISO)/. tests/fixtures/iso/
	@echo "refreshed tests/fixtures/iso from $(QUIRE_RS_ISO)"

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test deny deny-bans audit-unsafe audit-thin-boundary
