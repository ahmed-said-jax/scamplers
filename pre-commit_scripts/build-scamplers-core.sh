#!/usr/bin/env bash

cd rust/scamplers-core
wasm-pack build --release --out-dir ../../typescript/scamplers-core -- --features typescript
cp -r src ../../typescript/scamplers-core/rust-src
cd ../../typescript/scamplers-core
rm .gitignore
