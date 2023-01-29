all: clean run browse

run:
	cargo run

build:
	cargo build

clean:
	cargo clean

doc:
	cargo doc --no-deps --workspace --document-private-items

doc-all:
	cargo doc --workspace --document-private-items

browse:
	cargo doc --no-deps --workspace --document-private-items --open

web:
	wasm-pack build app/ --out-dir ../pkg --target web --dev

web-run:
	python3 -m http.server
