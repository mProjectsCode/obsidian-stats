.PHONY: all clean format lint build

DATA_ARGS := $(ARGS)

ifeq ($(FORCE),1)
DATA_ARGS += --force
endif

ifeq ($(NO_CLONE),1)
DATA_ARGS += --no-clone
endif

default: build

build:
	cargo run --release -- $(DATA_ARGS)
clean:
	cargo clean
format:
	cargo fmt
lint:
	cargo clippy
