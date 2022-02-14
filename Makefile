TMP_BUNDLE ?= /tmp/run0-bundle
BINARY := $(shell cat Cargo.toml | grep "name = " | sed 's/name = //g' | cut -d '"' -f2)

.PHONY: bundle build run0 run

# Helper to build run0
run0: src/*
	cargo build

# Helper to create a bundle.
# Simply call `make bundle` before running `make run`
bundle:
	./hack/mkbundle.sh $(TMP_BUNDLE)

# Helper to run run0.
# Requires that `make bundle` was executed before.
run: run0
	sudo ./target/debug/$(BINARY) run -b $(TMP_BUNDLE)
