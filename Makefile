SHELL := bash

.PHONY: test fmt

test:
	cargo test
fmt:
	cargo fmt