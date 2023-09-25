#!/bin/bash

if [ $# == 0 ]; then
  echo "please provide the source account and network, e.g.: --source alice --network futurenet"
  echo "note: the source identity used needs to created and funded to run this"
  exit 1
fi

set -ex

# make sure contract builds are up to date
soroban contract build

# make sure the native token wrapper contract exists (ignore error if it already exists)
soroban lab token wrap --asset native "$@" || true

# get the token contract address for the native token
TOKEN_ADDRESS=$(soroban lab token id --asset native "$@")
echo "export TOKEN_ADDRESS=$TOKEN_ADDRESS"

# deploy proxy
PROXY_ADDRESS=$(soroban contract deploy "$@" --wasm target/wasm32-unknown-unknown/release/relink_vrf_direct_funding_proxy.wasm)
echo "export PROXY_ADDRESS=$PROXY_ADDRESS"

# deploy consumer
CONSUMER_ADDRESS=$(soroban contract deploy "$@" --wasm target/wasm32-unknown-unknown/release/relink_vrf_consumer_example.wasm)
echo "export CONSUMER_ADDRESS=$CONSUMER_ADDRESS"

# initialize proxy
soroban contract invoke --id $PROXY_ADDRESS "$@" -- initialize --owner alice --token $TOKEN_ADDRESS
echo "# proxy initialized"

# add "backend" to the backend whitelist
soroban contract invoke --id $PROXY_ADDRESS "$@" -- add_backend_whitelist --address backend
echo "# identity \"backend\" added to proxy backend whitelist"

# initialize consumer
soroban contract invoke --id $CONSUMER_ADDRESS "$@" -- initialize --proxy $PROXY_ADDRESS --threshold 0 --oracles '[]'
echo "# consumer initialized"
