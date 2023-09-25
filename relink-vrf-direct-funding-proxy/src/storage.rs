use soroban_sdk::{contracttype, Address, Env};

use relink::{Error, RequestId};

const REQUEST_BUMP_AMOUNT: u32 = 34560; // 2 days

#[derive(Clone)]
#[contracttype]
pub enum DataKeyProxy {
    Token,
    Nonce,
    Request(RequestId),
    Whitelist(Address),
}

pub fn set_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKeyProxy::Token, &token);
}

pub fn get_token(env: &Env) -> Address {
    env.storage()
        .instance()
        .get::<_, Address>(&DataKeyProxy::Token)
        .expect("not initialized")
}

pub fn get_and_increment_nonce(env: &Env) -> u128 {
    // get value, default to zero
    let value = env
        .storage()
        .instance()
        .get::<_, u128>(&DataKeyProxy::Nonce)
        .unwrap_or(0);
    // store value increment by one
    env.storage()
        .instance()
        .set::<_, u128>(&DataKeyProxy::Nonce, &value.checked_add(1).unwrap());
    // return original value
    value
}

pub fn set_request_origin(env: &Env, id: RequestId, origin: &Address) {
    let key = DataKeyProxy::Request(id);
    env.storage().temporary().set(&key, origin);
    env.storage()
        .temporary()
        .bump(&key, REQUEST_BUMP_AMOUNT, REQUEST_BUMP_AMOUNT);
}

pub fn get_request_origin(env: &Env, id: RequestId) -> Result<Address, Error> {
    let key = DataKeyProxy::Request(id);
    env.storage()
        .temporary()
        .get(&key)
        .ok_or(Error::RequestUnknown)
}

pub fn remove_request_origin(env: &Env, id: RequestId) {
    let key = DataKeyProxy::Request(id);
    env.storage().temporary().remove(&key);
}

pub fn add_whitelist(env: &Env, origin: Address) {
    let key = DataKeyProxy::Whitelist(origin);
    env.storage().instance().set(&key, &());
}

pub fn remove_whitelist(env: &Env, origin: Address) {
    let key = DataKeyProxy::Whitelist(origin);
    env.storage().instance().remove(&key);
}

pub fn is_whitelisted(env: &Env, origin: Address) -> Result<(), Error> {
    let key = DataKeyProxy::Whitelist(origin);
    env.storage()
        .instance()
        .get(&key)
        .ok_or(Error::UnauthorizedBackend)
}
