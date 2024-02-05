use soroban_sdk::{contracttype, Address, Env};

use relink::{Error, RequestId};

const REQUEST_BUMP_AMOUNT: u32 = 34560; // 2 days

#[derive(Clone)]
#[contracttype]
pub enum DataKeyProxy {
    Token,
    Fee,
    Nonce,
    Whitelist(Address),
    RequestDapp(RequestId),
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

pub fn set_fee(env: &Env, fee: &i128) {
    env.storage().instance().set(&DataKeyProxy::Fee, fee);
}

pub fn get_fee(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get::<_, i128>(&DataKeyProxy::Fee)
        .unwrap()
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

pub fn add_whitelist(env: &Env, address: Address) {
    let key = DataKeyProxy::Whitelist(address);
    env.storage().instance().set(&key, &());
}

pub fn remove_whitelist(env: &Env, address: Address) {
    let key = DataKeyProxy::Whitelist(address);
    env.storage().instance().remove(&key);
}

pub fn is_whitelisted(env: &Env, address: Address) -> Result<(), Error> {
    let key = DataKeyProxy::Whitelist(address);
    env.storage()
        .instance()
        .get(&key)
        .ok_or(Error::UnauthorizedBackend)
}

pub fn set_request_dapp(env: &Env, id: RequestId, dapp: &Address) {
    let key = DataKeyProxy::RequestDapp(id);
    env.storage().temporary().set(&key, dapp);
    env.storage()
        .temporary()
        .extend_ttl(&key, REQUEST_BUMP_AMOUNT, REQUEST_BUMP_AMOUNT);
}

pub fn get_request_dapp(env: &Env, id: RequestId) -> Result<Address, Error> {
    let key = DataKeyProxy::RequestDapp(id);
    env.storage()
        .temporary()
        .get(&key)
        .ok_or(Error::RequestUnknown)
}

pub fn remove_request_dapp(env: &Env, id: RequestId) {
    let key = DataKeyProxy::RequestDapp(id);
    env.storage().temporary().remove(&key);
}
