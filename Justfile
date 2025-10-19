set positional-arguments
export RUST_BACKTRACE := "1"

kind := "dev"
profile_flag := "--profile " + kind

alias b := build
alias r := run
alias t := test
alias f := fix
alias u := update

default:
  @just --list

build:
  cargo build {{profile_flag}} --workspace

@run target *args="":
  #!/usr/bin/env sh
  shift; cargo run {{profile_flag}} -p {{target}} -- "$@"

@test *args="":
  #!/usr/bin/env sh
  cargo test --workspace --all-features --all-targets  -- --nocapture "$@"

fix:
  __CARGO_FIX_YOLO=1 cargo fix --workspace --all-features --all-targets --edition-idioms --broken-code
  __CARGO_FIX_YOLO=1 cargo clippy --workspace --all-targets --all-features --fix -Z unstable-options --broken-code
  cargo fmt --all
  pre-commit run --all-files

update:
  rustup update
  cargo install cargo-udeps cargo-edit
  # Need to use git repo, see https://github.com/killercup/cargo-edit/issues/869
  CARGO_REGISTRIES_CRATES_IO_PROTOCOL=git cargo fetch
  CARGO_REGISTRIES_CRATES_IO_PROTOCOL=git cargo upgrade --incompatible
  cargo update
  cargo build --workspace --all-features --all-targets
  pre-commit autoupdate

check:
  cargo check --workspace --all-features --all-targets
  cargo clippy --workspace --all-features --all-targets -- -D warnings
  cargo udeps --all-features --all-targets --workspace
  pre-commit run --all-files
