#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Vec};

use relink::{consumer, Error, EthAddress, RequestId, VrfDirectFundingConsumer};

mod events;
mod storage;

#[contract]
pub struct VrfDirectFundingConsumerExample;

#[contractimpl]
impl VrfDirectFundingConsumerExample {
    /// Initialize contract by setting the proxy address and trusted oracles.
    pub fn initialize(env: Env, proxy: Address, threshold: u32, oracles: Vec<EthAddress>) {
        consumer::init(&env, &proxy, threshold, oracles);
    }

    /// Initiate a request for randomness.
    pub fn initiate_randomness_request(env: Env, origin: Address, value: i128) -> RequestId {
        let id = consumer::request_randomness(&env, origin, value, None, None);
        // store the request id until the response is processed
        storage::add_request_id(&env, id.clone());
        id
    }
}

#[contractimpl]
impl VrfDirectFundingConsumer for VrfDirectFundingConsumerExample {
    /// Process the response to a randomness request.
    fn verify_and_fulfill_randomness(
        env: Env,
        id: RequestId,
        random_words: Vec<BytesN<32>>,
        signatures: Vec<(BytesN<64>, u32)>,
    ) -> Result<(), Error> {
        // check if RequestId exists
        storage::has_request_id(&env, id.clone())?;
        // verify signatures
        consumer::verify_randomness(&env, &id, &random_words, &signatures)?;
        // remove request as it should only be handled once
        storage::remove_request_id(&env, id.clone());
        // emit event containing the provided random words
        events::randomness_provided(&env, id, random_words);
        Ok(())
    }
}
