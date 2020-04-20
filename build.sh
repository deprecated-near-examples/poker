#!/bin/bash
cargo build -p poker --target wasm32-unknown-unknown --release
mkdir -p res
cp target/wasm32-unknown-unknown/release/poker.wasm ./res/
