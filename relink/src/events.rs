use soroban_sdk::{symbol_short, Address, Env};

use crate::EthAddress;

pub fn ownership_transfer_requested(env: &Env, from: &Address, to: &Address) {
    let topics = (symbol_short!("owner_req"), from, to);
    env.events().publish(topics, ());
}

pub fn ownership_transferred(env: &Env, from: &Address, to: &Address) {
    let topics = (symbol_short!("owner_acc"), from, to);
    env.events().publish(topics, ());
}

pub fn threshold_set(env: &Env, threshold: u32) {
    let topics = (symbol_short!("threshold"), threshold);
    env.events().publish(topics, ());
}

pub fn oracle_added(env: &Env, oracle: EthAddress) {
    let topics = (symbol_short!("oracle_ad"), oracle);
    env.events().publish(topics, ());
}

pub fn oracle_removed(env: &Env, oracle: EthAddress) {
    let topics = (symbol_short!("oracle_rm"), oracle);
    env.events().publish(topics, ());
}
