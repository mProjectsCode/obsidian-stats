.PHONY: all wasm data format lint submodule-update

DATA_MAKE_ARGS := ARGS="$(ARGS)" FORCE="$(FORCE)" NO_CLONE="$(NO_CLONE)" NO_RELEASE="$(NO_RELEASE)"

wasm:
	$(MAKE) -C data-wasm

data:
	$(MAKE) submodule-update
	$(MAKE) -C data build $(DATA_MAKE_ARGS)

format:
	$(MAKE) -C data format
	$(MAKE) -C data-lib format
	$(MAKE) -C data-wasm format
	cd website && bun run format

lint:
	$(MAKE) -C data lint
	$(MAKE) -C data-lib lint
	$(MAKE) -C data-wasm lint
	cd website && bun run check

submodule-update:
	git submodule update --init --remote
