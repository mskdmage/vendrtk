.PHONY = default lint test dev

default: lint test dev

lint:
	@cargo fmt &&\
	cargo clippy

# Tests

test:
	@cargo test --workspace

test.pipelines:
	@cargo test -p pipelines

# Targets

web.dev:
	@cargo watch -q -w crates/ -x "run -p webapp"