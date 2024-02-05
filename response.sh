#!/bin/bash

if [ $# == 0 ]; then
  echo "please provide the source account and network, e.g.: --network futurenet --source backend"
  echo "note: the source identity used needs to created and funded to run this"
  exit 1
fi

set -ex

# callback with randomness
soroban contract invoke \
  --id $PROXY_ADDRESS \
  --fee 1000000 \
  "$@" \
  -- callback_with_randomness \
    --backend backend \
    --id '["ce2cd94dbccd8fab617893437c996b8d551c18482b494a1081888a3909ba5a93"]' \
    --random_words '["d944a3fc3a228263ee8e8d9ea822280768d9d05d48b7e3961742fce43ba0f066"]' \
    --signatures '[]'

echo randomness provided
