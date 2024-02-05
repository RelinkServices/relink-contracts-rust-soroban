extern crate std;

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Vec};

use relink::{consumer, Error, EthAddress, RequestId, VrfDirectFundingConsumer};

const REQUEST_BUMP_AMOUNT: u32 = 34560; // 2 days

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Request(RequestId),
}

pub fn add_request_id(env: &Env, id: RequestId) {
    let key = DataKey::Request(id);
    env.storage().temporary().set(&key, &());
    env.storage()
        .temporary()
        .extend_ttl(&key, REQUEST_BUMP_AMOUNT, REQUEST_BUMP_AMOUNT);
}

pub fn has_request_id(env: &Env, id: RequestId) -> Result<(), Error> {
    let key = DataKey::Request(id);
    env.storage()
        .temporary()
        .get(&key)
        .ok_or(Error::RequestUnknown)
}

pub fn remove_request_id(env: &Env, id: RequestId) {
    let key = DataKey::Request(id);
    env.storage().temporary().remove(&key);
}

#[contract]
pub struct TestConsumer;

#[contractimpl]
impl TestConsumer {
    /// Initialize contract by setting the proxy address and trusted oracles.
    pub fn initialize(env: Env, proxy: Address, threshold: u32, oracles: Vec<EthAddress>) {
        consumer::init(&env, &proxy, threshold, oracles);
    }

    /// Initiate a request for randomness.
    pub fn initiate_randomness_request(env: Env, origin: Address, value: i128) -> RequestId {
        let id = consumer::request_randomness(&env, origin, value, None, None);
        // store the request id until the response is processed
        add_request_id(&env, id.clone());
        id
    }
}

#[contractimpl]
impl VrfDirectFundingConsumer for TestConsumer {
    /// Process the response to a randomness request.
    fn verify_and_fulfill_randomness(
        env: Env,
        id: RequestId,
        random_words: Vec<BytesN<32>>,
        signatures: Vec<(BytesN<64>, u32)>,
    ) -> Result<(), Error> {
        // check if RequestId exists
        has_request_id(&env, id.clone())?;
        // verify signatures
        consumer::verify_randomness(&env, &id, &random_words, &signatures)?;
        // remove request as it should only be handled once
        remove_request_id(&env, id.clone());
        Ok(())
    }
}
