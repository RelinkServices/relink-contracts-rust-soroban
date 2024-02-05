#!/bin/bash

if [ $# == 0 ]; then
  echo "please provide the source account and network, e.g.: --network futurenet --source bob"
  echo "note: the source identity used needs to created and funded to run this"
  exit 1
fi

set -ex

# initiate randomness request
soroban contract invoke \
  --id $CONSUMER_ADDRESS \
  --fee 10000000 \
  "$@" \
  -- initiate_randomness_request \
    --origin bob \
    --value 100

echo randomness requested
