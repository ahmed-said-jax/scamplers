#!/usr/bin/env bash

# Build Wasm module
cd rust/scamplers-core
wasm-pack build --release --out-dir ../../typescript/scamplers-core -- --features typescript

# Copy the source code for CI/CD to check
cd ../../typescript/scamplers-core
rm -r .gitignore rust-src
cp -r ../../rust/scamplers-core/src rust-src
