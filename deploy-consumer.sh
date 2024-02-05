#!/bin/bash

if [ $# == 0 ]; then
  echo "please provide the source account and network, e.g.: --network futurenet --source alice"
  echo "note: the source identity used needs to created and funded to run this"
  exit 1
fi

set -ex

# deploy consumer
CONSUMER_ADDRESS=$(soroban contract deploy "$@" --wasm target/wasm32-unknown-unknown/release/relink_vrf_direct_funding_consumer.optimized.wasm)
echo "export CONSUMER_ADDRESS=$CONSUMER_ADDRESS"

# initialize consumer
soroban contract invoke --id $CONSUMER_ADDRESS "$@" -- initialize --proxy $PROXY_ADDRESS --threshold 0 --oracles '[]'
echo "# consumer initialized"
