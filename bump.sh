#!/bin/bash

if [ $# == 0 ]; then
  echo "please provide the source account and network, e.g.: --network futurenet --source alice"
  echo "note: the source identity used needs to created and funded to run this"
  exit 1
fi

if [[ -z "${PROXY_ADDRESS}" ]]; then
  echo "missing env var: PROXY_ADDRESS"
  exit 1
fi

if [[ -z "${CONSUMER_ADDRESS}" ]]; then
  echo "missing env var: CONSUMER_ADDRESS"
  exit 1
fi

set -ex

# 518400 ledgers == 30 days
soroban contract extend "$@" --ledgers-to-extend 518400 --durability persistent --wasm target/wasm32-unknown-unknown/release/relink_vrf_direct_funding_proxy.optimized.wasm
soroban contract extend "$@" --ledgers-to-extend 518400 --durability persistent --wasm target/wasm32-unknown-unknown/release/relink_vrf_direct_funding_consumer.optimized.wasm
soroban contract extend "$@" --ledgers-to-extend 518400 --durability persistent --id $PROXY_ADDRESS
soroban contract extend "$@" --ledgers-to-extend 518400 --durability persistent --id $CONSUMER_ADDRESS
