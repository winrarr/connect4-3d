run:
    cargo run --release

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

test:
    cargo test --all-targets

lint:
    cargo clippy --all-targets -- -D warnings

build:
    cargo build --release

verify: fmt-check test lint
