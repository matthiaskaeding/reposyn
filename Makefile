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

# Test setup repos
RUST_REPO_URL = https://github.com/rust-lang/rust.git
RUST_FOLDER = tests/repos/rust
RUST_COMMIT = d2f335d58e6c346f94910d0f49baf185028b44be
test-setup-rust:
	@echo "Setting up test repository..."
	@mkdir -p tests/repos
	@if [ ! -d "$(RUST_FOLDER)" ]; then \
		cd tests/repos && \
		git clone $(RUST_REPO_URL) rust && \
		cd rust && \
		git checkout $(RUST_COMMIT); \
	fi
RIPGREP_REPO_URL = https://github.com/BurntSushi/ripgrep.git
RIPGREP_FOLDER = test-repos/ripgrep
RIPGREP_COMMIT = e2362d4d5185d02fa857bf381e7bd52e66fafc73
test-setup-ripgrep:
	@echo "Setting up test repository..."
	@mkdir -p tests/repos
	@if [ ! -d "$(RIPGREP_FOLDER)" ]; then \
		cd tests/repos && \
		git clone $(RIPGREP_REPO_URL) ripgrep && \
		cd ripgrep && \
		git checkout $(RIPGREP_COMMIT); \
	fi
