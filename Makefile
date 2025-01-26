get-requests:
	git clone https://github.com/psf/requests.git

build-release:
	cargo build --release
run:
	cargo run --quiet
run: build run

.PHONY: build run run get-requests
