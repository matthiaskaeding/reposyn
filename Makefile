get-requests:
	git clone https://github.com/psf/requests.git

build-release:
	cargo build --release
run:
	cargo run --quiet
reqs:
	uv pip install -r requirements.txt
.PHONY: build run run get-requests
