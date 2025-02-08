#!/bin/bash

# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Create a gzipped version with maximum compression
gzip -9 -c target/wasm32-unknown-unknown/release/ooga_booga_contract.wasm > contract.wasm.gz