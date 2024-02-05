use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, Vec};

use relink::RequestId;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[contracttype]
pub struct RandomnessRequestData {
    pub id: RequestId,
    pub request_confirmations: u32,
    pub num_words: u32,
}

/// Equivalent event to the one in Solidity:
/// event RandomnessRequest(
///     address indexed user,
///     address indexed dapp,
///     uint256 indexed nonce,
///     bytes32 requestId,
///     uint16 _requestConfirmations,
///     uint32 _numWords
/// );
pub(crate) fn randomness_requested(
    env: &Env,
    origin: Address,
    dapp: Address,
    nonce: u128,
    id: RequestId,
    request_confirmations: u32,
    num_words: u32,
) {
    let topics = (symbol_short!("request"), origin, dapp, nonce);
    env.events().publish(
        topics,
        RandomnessRequestData {
            id,
            request_confirmations,
            num_words,
        },
    );
}

pub(crate) fn randomness_provided(env: &Env, id: RequestId, random_words: Vec<BytesN<32>>) {
    let topics = (symbol_short!("response"), id);
    env.events().publish(topics, random_words);
}

pub(crate) fn fee_set(env: &Env, fee: i128) {
    let topics = (symbol_short!("fee_set"), fee);
    env.events().publish(topics, ());
}

pub(crate) fn whitelist_address_added(env: &Env, address: Address) {
    let topics = (symbol_short!("wl_add"), address);
    env.events().publish(topics, ());
}

pub(crate) fn whitelist_address_removed(env: &Env, address: Address) {
    let topics = (symbol_short!("wl_rm"), address);
    env.events().publish(topics, ());
}
