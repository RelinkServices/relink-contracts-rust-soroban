#!/bin/bash

set -ex

source .soroban/testnet.env

# callback with randomness
soroban contract invoke \
  --id $PROXY_ADDRESS \
  --network testnet \
  --source backend \
  --fee 1000000 \
  -- callback_with_randomness \
    --backend backend \
    --request_origin alice \
    --id '["0ed506e5adb9261ac31b003b5468833335267952aa717329f0f77f7d94504c8b"]' \
    --random_words '["d944a3fc3a228263ee8e8d9ea822280768d9d05d48b7e3961742fce43ba0f066"]' \
    --signatures '{}'

echo randomness provided
