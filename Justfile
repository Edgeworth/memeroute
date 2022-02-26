export RUST_BACKTRACE := "1"
export RUST_LOG := "info"
kind := "debug"
profile_flag := if kind == "release" { "--release "} else { "" }

alias b := build
alias r := run
alias t := test
alias f := fix
alias u := update

default:
  @just --list

build:
  cargo build {{profile_flag}} --workspace

run target *args="":
  cargo run {{profile_flag}} -p {{target}} -- {{args}}

test:
  cargo test --workspace --all-features --all-targets  -- --nocapture

fix:
  __CARGO_FIX_YOLO=1 cargo fix --workspace --all-features --all-targets --edition-idioms --broken-code
  __CARGO_FIX_YOLO=1 cargo clippy --workspace --all-targets --all-features --fix -Z unstable-options --broken-code
  cargo fmt --all
  cargo udeps --all-features --all-targets --workspace

update:
  rustup update
  cargo install cargo-udeps cargo-edit
  cargo upgrade --workspace
  cargo update
  cargo build --workspace --all-features --all-targets
  pre-commit autoupdate
  SETUPTOOLS_USE_DISTUTILS=stdlib pre-commit run --all-files
