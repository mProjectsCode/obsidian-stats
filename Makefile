.PHONY: all wasm data format lint submodule-update

wasm:
	cd data-wasm && make

data:
	make submodule-update
	cd data && make

format:
	cd data && make format
	cd data-lib && make format
	cd data-wasm && make format
	cd website && bun run format

lint:
	cd data && make lint
	cd data-lib && make lint
	cd data-wasm && make lint

submodule-update:
	git submodule update --init --remote