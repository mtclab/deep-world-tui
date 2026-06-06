.PHONY: check build test clippy fmt clean

check: fmt clippy build test

fmt:
	cargo fmt --check

clippy:
	cargo clippy --all-targets -- -D warnings

build:
	cargo build

build-no-llm:
	cargo build --no-default-features

test:
	cargo test

clean:
	cargo clean