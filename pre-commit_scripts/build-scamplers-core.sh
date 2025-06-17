#!/usr/bin/env bash

set -euo pipefail

cd rust/scamplers-core
wasm-pack build --release --out-dir ../../typescript/scamplers-core -- --features typescript
cd ../../typescript/scamplers-core
rm -r .gitignore rust-src
cp -r ../../rust/scamplers-core/src rust-src
