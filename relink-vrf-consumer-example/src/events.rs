use soroban_sdk::{symbol_short, BytesN, Env, Vec};

use relink::RequestId;

pub(crate) fn randomness_provided(env: &Env, id: RequestId, random_words: Vec<BytesN<32>>) {
    let topics = (symbol_short!("rand_recv"), id);
    env.events().publish(topics, random_words);
}
