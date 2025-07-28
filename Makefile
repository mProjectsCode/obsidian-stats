wasm:
	cd data-wasm && make

data:
	cd data && make

format:
	cd data && make format
	cd data-lib && make format
	cd data-wasm && make format

submodule-update:
	git submodule update --init --remote