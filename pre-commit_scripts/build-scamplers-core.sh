#!/usr/bin/env bash

cd rust/scamplers-core
wasm-pack build --release --out-dir ../../typescript/scamplers-core -- --features typescript
cp -r src ../../typescript/scamplers-core
cd ../../typescript/scamplers-core
mv src rust-src
rm .gitignore
