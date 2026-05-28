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
# Fixtures
# =============================================================================

QUIRE_RS_ISO ?= ../quire-rs/tests/render_parity/modules/iso

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
# Benchmarks
# =============================================================================

BENCH_MODULE ?= $(CURDIR)/tests/fixtures/iso
BENCH_CTX    ?= $(CURDIR)/tests/fixtures/contexts/FR.json
BENCH_P95_MS ?= 50

.PHONY: bench
bench: build
	@command -v hyperfine >/dev/null || { echo "hyperfine not installed" >&2; exit 1; }
	hyperfine --warmup 3 --export-json target/bench.json \
		'target/release/quire render FR --module $(BENCH_MODULE) --data $(BENCH_CTX)'
	@python3 scripts/check_bench_p95.py target/bench.json $(BENCH_P95_MS)

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test deny deny-bans audit-unsafe audit-thin-boundary
