#!/bin/bash
echo Deleting poker contract
near delete poker node0
echo Creating poker account
near create_account poker --masterAccount node0 --initialBalance 1000 --keyPath ~/.near/localnet/node0/validator_key.json
./scripts/deploy.sh
