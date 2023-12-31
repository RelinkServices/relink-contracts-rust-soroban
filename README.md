# Relink - Trustless Chainlink Relaying Service

Relink empowers new blockchains, or those with less focus, to access established services on the major chains.

Chainlink itself supports only a few chains at all. At the same time, the speed at which new chains are deployed is constantly increasing. Be it as L2s or even L3s or as another competitive L1s.

Relink has developed a proxy service solution that forwards the random data request to an officially from Chainlink supported chain and processes the result accordingly.

A Relink oracle network signs the data generated by Chainlink and passes the original data along with the signatures to the callback method. The signature checks in the consumer base contract mean that no data can be changed on the way from the relaying backend.

## Getting Started

### Configuration

Relink Proxy Addresses:

- Soroban Mainnet: `not yet available`
- Soroban Testnet: `CBCMN5XB4CL5VQKPYJ3QTZAKGY3GO26YSTTZROMWETUBB3TME3JBRAOD`
- Soroban Futurenet: `CD3L2XQIVM65RQC27PSXVWJST7LNHKGNSZZJDITT65TWU3SIAQMRZ54H`

### Prerequisite

1. Setup Soroban: https://soroban.stellar.org/docs/getting-started/setup
2. Run `soroban contract build`
3. Generate identity: `soroban config identity generate alice` to generate an identity called "alice". Keypair will be stored in `.soroban` folder
4. Add testnet config to CLI `soroban config network add testnet --rpc-url "https://horizon-testnet.stellar.org" --network-passphrase "Test SDF Network ; September 2015"`

### Deploy Consumer Example

```
soroban contract deploy \
 --network futurenet \
 --source alice \
 --wasm target/wasm32-unknown-unknown/release/relink_vrf_consumer_example.wasm
```

The consumer contract address (`$CONSUMER_CONTRACT_ID`) will be printed in the console after deployment.

```
soroban contract invoke
--id $CONSUMER_CONTRACT_ID
--network futurenet
--source alice
-- initialize
--proxy $PROXY_ADDRESS
--threshold 0
--oracles '[]'
```

### Initiate Consumer Example Randomness Request

Stellar Soroban Futurenet:

```
soroban contract invoke \
 --id $CONSUMER_ADDRESS \
 --network futurenet \
 --source alice \
 --fee 1000000 \
 -- initiate_randomness_request \
 --origin alice \
 --value 10
```

## Example

### Stellar Soroban Futurenet

- `InitiateRandomnessRequest` in [this transaction](https://futurenet.steexp.com/tx/49b98f566920b42b77897f78067f9c99d02cff1e6581a8637b8975470ee67da4) on Soroban Futurenet
- `RequestRandomWords` in [this transaction](https://polygonscan.com/tx/0x7f9778c24ecd2f86f7efaf7fd49f5abf2dd911c4b82a0f4d2a584b4f766d9e34#eventlog) on Polygon
- `RandomnessReceived` in [this transaction](https://polygonscan.com/tx/0x736b7a6a4b5e24b13e7ff43f4d5a690a8c829b5acc9c79fa108c77040743af86#eventlog) on Polygon
- `CallbackWithRandomness` in [this transaction](https://futurenet.steexp.com/tx/16a8d4a602b58c3da35bb4fe7cd92581580a18abff14a079834d7072a5a7540a) on Soroban Futurenet

## Build

```bash
soroban contract build
```

## Test on local node

Locally run a standalone node

```bash
docker run --rm -it --platform linux/amd64 -p 8000:8000 --name stellar stellar/quickstart:soroban-dev --standalone --enable-soroban-rpc
```

Make sure you have an identity called `alice` that has some funds:

```bash
soroban config identity generate alice
curl "http://localhost:8000/friendbot?addr=$(soroban config identity address alice)"
```

Run integration test and source the results

```bash
./test.sh
source .soroban/test.env
```

Read the contracts balance

```bash
soroban contract invoke --id $TOKEN_ADDRESS --source alice --network standalone -- balance --id $PROXY_ADDRESS
```
