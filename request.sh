#!/bin/bash

set -ex

source .soroban/testnet.env

# initiate randomness request
soroban contract invoke \
  --id $CONSUMER_ADDRESS \
  --network testnet \
  --source alice \
  --fee 1000000 \
  -- initiate_randomness_request \
    --origin alice \
    --value 10

echo randomness requested
