get-requests:
	git clone https://github.com/psf/requests.git

build:
	cargo build
run:
	cargo run --quiet
brun: build run

.PHONY: build run brun get-requests
