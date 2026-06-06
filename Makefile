.PHONY: test build install uninstall release-check

test:
	cargo test

build:
	cargo build --release

install:
	./scripts/install.sh

uninstall:
	./scripts/uninstall.sh

release-check:
	cargo fmt --check
	cargo check
	cargo test
	cargo build --release
