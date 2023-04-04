all: clean run browse

run:
	cargo run

build:
	cargo build

debug:
	RUST_BACKTRACE=1 cargo run

clean:
	cargo clean

doc:
	cargo doc --no-deps --workspace --document-private-items

doc-all:
	cargo doc --workspace --document-private-items

browse:
	cargo doc --no-deps --workspace --document-private-items --open

loc:
	loc app
	loc gfx-api
	loc gfx-wgpu
	loc tool
	loc utils
	loc world
	loc world-api
	loc

# use to print layout of types
# cargo +nightly rustc -- -Zprint-type-sizes
