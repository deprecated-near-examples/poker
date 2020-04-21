#!/bin/bash
rustup target add wasm32-unknown-unknown
cargo build -p poker --target wasm32-unknown-unknown --release
mkdir -p res
cp target/wasm32-unknown-unknown/release/poker.wasm ./res/
