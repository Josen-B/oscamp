#!/bin/bash
set -e

echo "Building skernel-x86..."

# Build the project
cargo build --release --target x86_64-unknown-none

# The binary will be at target/x86_64-unknown-none/release/skernel-x86
# Copy it to a more convenient location
cp target/x86_64-unknown-none/release/skernel-x86 skernel

echo "Build complete: skernel"
