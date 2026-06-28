.PHONY = default lint test dev

default: lint test dev

lint:
	@cargo fmt &&\
	cargo clippy

test:
	@cargo test --workspace

# Targets

web.dev:
	@cargo watch -q -w crates/ -x "run -p webapp"