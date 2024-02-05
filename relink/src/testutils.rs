#![cfg(any(test, feature = "testutils"))]
extern crate alloc;

use alloc::format;
use alloc::string::String;

use ethsign::{PublicKey, SecretKey};
use hex_slice::AsHex;
use rand::rngs::mock::StepRng;
use rand::RngCore;
use soroban_sdk::testutils::arbitrary::std;
use soroban_sdk::testutils::Ledger;
use soroban_sdk::unwrap::UnwrapOptimized;
use soroban_sdk::{token, Address, Bytes, BytesN, Env, Vec};

use crate::{EthAddress, RequestId};

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
    hash_hex(&env.crypto().keccak256(input))
}

pub fn hash_u8(env: &Env, input: &[u8]) -> String {
    hash_bytes(env, &Bytes::from_slice(env, input))
}

pub struct TestOracle {
    secret: SecretKey,
    public: PublicKey,
}

pub struct TestOracleGenerator {
    rng: StepRng,
}

impl TestOracleGenerator {
    pub fn new() -> Self {
        Self {
            // arbitrary numbers so that generated keys are deterministic and tests results are reproducible
            rng: StepRng::new(756124321, 12836213),
        }
    }

    pub fn generate_sorted(&mut self, env: &Env, count: u32) -> std::vec::Vec<TestOracle> {
        let mut raw = [0u8; 32];
        let mut oracles: std::vec::Vec<TestOracle> = std::vec::Vec::new();
        for _ in 0..count {
            self.rng.fill_bytes(&mut raw);
            oracles.push(TestOracle::new(&raw));
        }
        oracles.sort_by_key(|oracle| oracle.address(env));
        oracles
    }
}

impl TestOracle {
    pub fn new(raw: &[u8; 32]) -> Self {
        let secret = SecretKey::from_raw(raw).unwrap();
        let public = secret.public();
        Self { secret, public }
    }

    pub fn address(&self, env: &Env) -> EthAddress {
        let raw_bytes = self.public.address();
        let pub_key = BytesN::from_array(&env, raw_bytes);
        EthAddress::from_bytes(pub_key)
    }

    pub fn sign(
        &self,
        env: &Env,
        consumer_address: &Address,
        id: &RequestId,
        random_words: &Vec<BytesN<32>>,
    ) -> (BytesN<64>, u32) {
        // generate message-to-sign
        let domain_separator = crate::consumer::domain_separator(&env, consumer_address);
        let tx_input_hash = crate::consumer::tx_input_hash(&env, id, &random_words);
        // create message-to-sign
        // bytes32 totalHash = keccak256(
        //     abi.encodePacked("\x19\x01", domainSeparator, txInputHash)
        // );
        let mut msg = Bytes::from_array(env, &[0x19, 0x01]);
        msg.append(domain_separator.as_ref());
        msg.append(tx_input_hash.as_ref());
        let msg_digest: [u8; 32] = env.crypto().keccak256(&msg).to_array();
        let signature = self.secret.sign(&msg_digest).unwrap();
        let mut signature_bytes = Bytes::new(env);
        signature_bytes.append(&Bytes::from_array(env, &signature.r));
        signature_bytes.append(&Bytes::from_array(env, &signature.s));
        (
            signature_bytes.try_into().unwrap_optimized(),
            signature.v as u32,
        )
    }
}

#[cfg(test)]
mod test {
    use soroban_sdk::testutils::arbitrary::std::println;
    use soroban_sdk::Env;

    use crate::testutils::TestOracleGenerator;

    #[test]
    fn address_sort() {
        let env = Env::default();
        let mut generator = TestOracleGenerator::new();
        let oracles = generator.generate_sorted(&env, 100);
        println!("sorted oracle addresses:");
        for oracle in oracles {
            println!("{}", oracle.address(&env));
        }
    }
}
