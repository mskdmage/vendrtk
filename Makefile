.PHONY=default lint test server.run server.dev server.build

default: lint test server.dev

lint:
	@cargo fmt &&\
	cargo clippy

test:
	@cargo test --workspace

# Binary targets

## Web server

server.dev:
	@cargo watch -q -w crates/bin/vendrtk-server -x "run -p vendrtk-server"

server.test:
	@cargo test -p vendrtk-server

server.build:
	@cargo build -p vendrtk-server --release