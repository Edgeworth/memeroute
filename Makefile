gui:
	RUST_BACKTRACE=1 RUST_LOG=info cargo run -p memeroute-gui

test:
	RUST_BACKTRACE=1 cargo test --workspace --all-features --all-targets  -- --nocapture

fix:
	cargo fix --workspace --all-features --all-targets --edition-idioms
	cargo clippy --workspace --all-targets --all-features --fix -Z unstable-options
	cargo fmt --all
	cargo outdated --workspace
	cargo udeps --all-features --all-targets --workspace

update-deps:
	rustup update
	cargo install cargo-audit cargo-outdated cargo-bloat cargo-tree cargo-udeps
	cargo update
	cargo build --workspace --all-features --all-targets
