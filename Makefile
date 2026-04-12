# Rudra Office — Build & Development

CARGO := cargo
WASM_PACK := wasm-pack
WASM_CRATE := ffi/wasm
WASM_OUT := web/wasm-pkg

.PHONY: help build test clippy fmt wasm wasm-release server clean

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# ─── Rust Engine ──────────────────────────────────────

build: ## Build all crates (debug)
	$(CARGO) build --workspace

test: ## Run all tests
	$(CARGO) test --workspace --exclude s1engine-wasm --exclude s1engine-c

clippy: ## Run clippy lints
	$(CARGO) clippy --workspace -- -D warnings

fmt: ## Check formatting
	$(CARGO) fmt --all --check

check: clippy fmt test ## Run all checks

# ─── WASM ─────────────────────────────────────────────

wasm: ## Build WASM (dev mode)
	bash scripts/build-wasm.sh --dev
	@mkdir -p $(WASM_OUT)
	cp demo/pkg/*.js demo/pkg/*.wasm demo/pkg/*.ts $(WASM_OUT)/ 2>/dev/null || true

wasm-release: ## Build WASM (release mode)
	bash scripts/build-wasm.sh
	@mkdir -p $(WASM_OUT)
	cp demo/pkg/*.js demo/pkg/*.wasm demo/pkg/*.ts $(WASM_OUT)/ 2>/dev/null || true

# ─── Server ───────────────────────────────────────────

server: ## Run the Axum API server
	$(CARGO) run -p s1-server

relay: ## Run the WebSocket relay
	node scripts/relay.js

# ─── Docker ───────────────────────────────────────────

docker-build: ## Build Docker image
	docker build -t rudra-office .

docker-run: ## Run Docker container
	docker run -p 8787:8787 rudra-office

# ─── Clean ────────────────────────────────────────────

clean: ## Clean build artifacts
	$(CARGO) clean
	rm -rf $(WASM_OUT)
