#!/usr/bin/env bash
set -euo pipefail

rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
echo "Built target/x86_64-pc-windows-gnu/release/forza-key-overlay.exe"
