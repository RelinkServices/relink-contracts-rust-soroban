use soroban_sdk::{contracttype, Env};

use relink::{Error, RequestId};

pub(crate) const REQUEST_TTL: u32 = 34560; // 2 days

#[derive(Clone)]
#[contracttype]
pub enum DataKeyConsumerExample {
    Request(RequestId),
}

pub fn add_request_id(env: &Env, id: RequestId) {
    let key = DataKeyConsumerExample::Request(id);
    env.storage().temporary().set(&key, &());
    env.storage()
        .temporary()
        .extend_ttl(&key, REQUEST_TTL, REQUEST_TTL);
}

pub fn has_request_id(env: &Env, id: RequestId) -> Result<(), Error> {
    let key = DataKeyConsumerExample::Request(id);
    env.storage()
        .temporary()
        .get(&key)
        .ok_or(Error::RequestUnknown)
}

pub fn remove_request_id(env: &Env, id: RequestId) {
    let key = DataKeyConsumerExample::Request(id);
    env.storage().temporary().remove(&key);
}
