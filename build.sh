#! /bin/bash

# Depends on Node.js v20 or newer, npm and Rust
cd parametric-renderer-core
cargo run --bin copy-includes
# Can also run arbitrary code, because of npm postinstall hooks
npm ci
npm run build

cd ..
npm ci
npm run build
