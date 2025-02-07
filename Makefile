
get-repos:
	git clone https://github.com/psf/requests.git

build-release:
	cargo build --release
run:
	cargo run --quiet

.PHONY: build run run get-requests

lint:
	cargo fmt --all -- --check
	cargo clippy -- -D warnings
