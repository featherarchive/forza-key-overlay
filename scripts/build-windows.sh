#!/usr/bin/env bash
set -euo pipefail

rustup target add x86_64-pc-windows-msvc

if ! command -v cargo-xwin >/dev/null 2>&1; then
  cargo install cargo-xwin --locked
fi

cargo xwin build --release --target x86_64-pc-windows-msvc
mkdir -p dist
cp target/x86_64-pc-windows-msvc/release/forza-key-overlay.exe dist/forza-key-overlay.exe
echo "Built dist/forza-key-overlay.exe"
