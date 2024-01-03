#!/bin/bash

cd "$(dirname "$0")" || exit

# https://doc.rust-lang.org/nightly/rustc/platform-support.html
brew install zig
cargo install cargo-zigbuild

cargo clean --release

# Linux x86_64: warp-guard-linux-x86_64
rustup target add x86_64-unknown-linux-musl
cargo zigbuild --release --target x86_64-unknown-linux-musl || exit
