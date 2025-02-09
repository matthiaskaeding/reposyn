# Repository URLs and configurations
RUST_REPO_URL := https://github.com/rust-lang/rust.git
RUST_FOLDER := tests/repos/rust
RUST_COMMIT := d2f335d58e6c346f94910d0f49baf185028b44be

RIPGREP_REPO_URL := https://github.com/BurntSushi/ripgrep.git
RIPGREP_FOLDER := tests/repos/ripgrep
RIPGREP_COMMIT := e2362d4d5185d02fa857bf381e7bd52e66fafc73

# Phony targets declaration
.PHONY: get-repos build-release run lint clean test-setup-rust test-setup-ripgrep run-rust

# Main targets
get-repos:
	git clone https://github.com/psf/requests.git

build-release:
	cargo build --release

run:
	cargo run --quiet

lint:
	cargo fmt --all -- --check
	cargo clippy -- -D warnings

clean:
	rm -f repo-synopsis.txt

# Test repository setup
test-setup-rust:
	@echo "Setting up Rust test repository..."
	@mkdir -p tests/repos
	@if [ ! -d "$(RUST_FOLDER)" ]; then \
		cd tests/repos && \
		git clone $(RUST_REPO_URL) rust && \
		cd rust && \
		git checkout $(RUST_COMMIT); \
	fi

test-setup-ripgrep:
	@echo "Setting up Ripgrep test repository..."
	@mkdir -p tests/repos
	@if [ ! -d "$(RIPGREP_FOLDER)" ]; then \
		cd tests/repos && \
		git clone $(RIPGREP_REPO_URL) ripgrep && \
		cd ripgrep && \
		git checkout $(RIPGREP_COMMIT); \
	fi

run-rust:
	@echo "Running reposyn on rust repo. Takes about 10 seconds"
	cargo run -- -f tests/repos/rust
