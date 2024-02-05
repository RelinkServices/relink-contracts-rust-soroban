#!/bin/bash

set -ex

# make sure contract builds are up to date
soroban contract build
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/relink_vrf_direct_funding_proxy.wasm
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/relink_vrf_direct_funding_consumer.wasm
