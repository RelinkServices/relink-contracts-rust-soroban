#![no_std]

use soroban_sdk::{contractclient, contracterror, Address, BytesN, Env, Vec};

pub use confirmed_owner::ConfirmedOwner;
pub use eth_address::EthAddress;
pub use request_id::RequestId;

pub mod confirmed_owner;
pub mod consumer;
mod eth_address;
mod events;
mod request_id;
pub mod testutils;
pub mod utils;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    RequestUnknown = 1,
    InsufficientFee = 2,
    UnauthorizedBackend = 3,
    TooFewSignatures = 4,
    UnauthorizedOracleSignatures = 5,
    UnorderedOracles = 6,
}

#[contractclient(name = "VrfDirectFundingProxyClient")]
pub trait VrfDirectFundingProxy {
    /// Generate a new request for randomness.
    fn request_randomness(
        env: Env,
        origin: Address,
        value: i128,
        dapp: Address,
        request_confirmations: u32,
        num_words: u32,
    ) -> Result<RequestId, Error>;

    /// Entry point for randomness data coming from a backend.
    fn callback_with_randomness(
        env: Env,
        backend: Address,
        id: RequestId,
        random_words: Vec<BytesN<32>>,
        signatures: Vec<(BytesN<64>, u32)>,
    ) -> Result<(), Error>;
}

#[contractclient(name = "VrfDirectFundingConsumerClient")]
pub trait VrfDirectFundingConsumer {
    fn verify_and_fulfill_randomness(
        env: Env,
        id: RequestId,
        random_words: Vec<BytesN<32>>,
        signatures: Vec<(BytesN<64>, u32)>,
    ) -> Result<(), Error>;
}
