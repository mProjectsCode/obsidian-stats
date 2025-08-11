.PHONY: all clean format lint build

default: build

build:
	wasm-pack build --target web
clean:
	cargo clean
format:
	cargo fmt
lint:
	cargo clippy