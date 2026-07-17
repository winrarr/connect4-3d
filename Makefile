CARGO ?= cargo

.PHONY: run fmt fmt-check test lint build verify

run:
	$(CARGO) run --release

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

test:
	$(CARGO) test --all-targets

lint:
	$(CARGO) clippy --all-targets -- -D warnings

build:
	$(CARGO) build --release

verify: fmt-check test lint build
