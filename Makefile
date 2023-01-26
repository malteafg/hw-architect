all: clean run browse

run:
	cargo run

build:
	cargo build

clean:
	cargo clean

doc:
	cargo doc --no-deps --workspace --document-private-items

browse:
	cargo doc --no-deps --workspace --document-private-items --open

