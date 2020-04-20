#!/bin/bash
echo Deploying deck contract to poker
near deploy --wasmFile res/poker.wasm  --accountId poker --keyPath neardev/local/poker.json
