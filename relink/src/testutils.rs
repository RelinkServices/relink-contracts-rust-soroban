#![cfg(any(test, feature = "testutils"))]
extern crate alloc;

use alloc::format;
use alloc::string::String;

use ed25519_dalek::{Signer, SigningKey};
use hex_slice::AsHex;
use rand::thread_rng;
use soroban_sdk::testutils::Ledger;
use soroban_sdk::{token, Address, Bytes, BytesN, Env, IntoVal};

pub fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract(admin.clone());
    (
        token::Client::new(env, &contract_address),
        token::StellarAssetClient::new(env, &contract_address),
    )
}

pub fn advance_ledger(env: &Env, delta: u64) {
    env.ledger().with_mut(|l| {
        l.timestamp += delta;
    });
}

pub fn hash_hex(hash: &BytesN<32>) -> String {
    format!("0x{:02x}", hash.to_array().plain_hex(false))
}

pub fn hash_bytes(env: &Env, input: &Bytes) -> String {
    hash_hex(&env.crypto().sha256(input))
}

pub fn hash_u8(env: &Env, input: &[u8]) -> String {
    hash_hex(&env.crypto().sha256(&Bytes::from_slice(env, input)))
}

pub struct TestSigner {
    key: SigningKey,
}

impl TestSigner {
    pub fn new() -> Self {
        Self {
            key: SigningKey::generate(&mut thread_rng()),
        }
    }

    pub fn pub_key(&self, env: &Env) -> BytesN<32> {
        self.key.verifying_key().to_bytes().into_val(env)
    }

    pub fn sign(&self, env: &Env, msg: &[u8]) -> BytesN<64> {
        self.key.sign(msg).to_bytes().into_val(env)
    }
}
