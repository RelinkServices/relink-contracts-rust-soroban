#!/bin/bash

set -e

# make sure contract builds are up to date
soroban contract build

# remove generated bindings
rm -rf bindings/relink-vrf-direct-funding-proxy-client

# rebuild bindings
soroban contract bindings typescript \
  --network futurenet \
  --output-dir bindings/relink-vrf-direct-funding-proxy-client \
  --contract-id CBXLM46D3IYZNYKNE3T3LRRDMLH6EN6JB6QIDO2S5RYNL6CRZCYWOP2F \
  --wasm target/wasm32-unknown-unknown/release/relink_vrf_direct_funding_proxy.wasm
