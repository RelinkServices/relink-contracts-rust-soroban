use core::fmt::{Display, Formatter};

use hex_slice::AsHex;
use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env};

use crate::utils::address_bytes;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[contracttype]
pub struct RequestId(BytesN<32>);

impl RequestId {
    pub fn new(env: &Env, origin: &Address, dapp: &Address, proxy: &Address, nonce: u128) -> Self {
        let mut buffer = Bytes::new(env);
        buffer.append(&env.ledger().network_id().as_ref());
        buffer.append(&address_bytes(env, &origin));
        buffer.append(&address_bytes(env, &dapp));
        buffer.append(&address_bytes(env, &proxy));
        buffer.append(&Bytes::from_array(env, &nonce.to_be_bytes()));
        RequestId(env.crypto().keccak256(&buffer))
    }

    pub fn zero(env: &Env) -> Self {
        RequestId(BytesN::from_array(env, &[0; 32]))
    }

    pub fn from_bytes(bytes: BytesN<32>) -> Self {
        RequestId(bytes)
    }

    pub fn as_bytes(&self) -> &BytesN<32> {
        &self.0
    }
}

impl Display for RequestId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "0x{:02x}", self.0.to_array().plain_hex(false))
    }
}

#[cfg(test)]
mod test {
    extern crate alloc;

    use alloc::format;

    use soroban_sdk::testutils::arbitrary::std::println;
    use soroban_sdk::testutils::Address as AddressTestTrait;
    use soroban_sdk::{Address, BytesN, Env};

    use crate::RequestId;

    #[test]
    fn hex() {
        let env = Env::default();
        let address = Address::generate(&env);
        let id = RequestId::new(&env, &address, &address, &address, 0);
        assert_eq!(
            format!("{}", id),
            "0x954919074d57999f740bf019cc0de0614cd52b1d8a1bac78b4efba6bec1216d4"
        );
    }

    #[test]
    fn empty() {
        let env = Env::default();
        let strkey = stellar_strkey::Contract([0u8; 32]);
        println!("Empty Address: {strkey}");
        // convert strkey to address
        let address = Address::from_string(&soroban_sdk::String::from_str(
            &env,
            strkey.to_string().as_str(),
        ));
        let id = RequestId::new(&env, &address, &address, &address, 0);
        assert_eq!(
            id,
            RequestId::from_bytes(BytesN::from_array(
                &env,
                &[
                    205, 46, 102, 191, 11, 145, 238, 237, 198, 198, 72, 174, 147, 53, 167, 141,
                    124, 154, 74, 176, 239, 51, 97, 42, 130, 77, 145, 205, 198, 138, 79, 33
                ]
            ))
        );
    }

    #[test]
    fn random() {
        let env = Env::default();
        let foo = Address::generate(&env);
        let bar = Address::generate(&env);
        for nonce in 0..100 {
            // same inputs should give the same ID
            assert_eq!(
                RequestId::new(&env, &foo, &foo, &foo, nonce),
                RequestId::new(&env, &foo, &foo, &foo, nonce),
            );
            // different nonce should give different ID
            assert_ne!(
                RequestId::new(&env, &foo, &foo, &foo, nonce),
                RequestId::new(&env, &foo, &foo, &foo, nonce + 1),
            );
            // different address in any of the inputs should give different ID
            assert_ne!(
                RequestId::new(&env, &foo, &foo, &foo, nonce),
                RequestId::new(&env, &bar, &foo, &foo, nonce),
            );
            assert_ne!(
                RequestId::new(&env, &foo, &foo, &foo, nonce),
                RequestId::new(&env, &foo, &bar, &foo, nonce),
            );
            assert_ne!(
                RequestId::new(&env, &foo, &foo, &foo, nonce),
                RequestId::new(&env, &foo, &foo, &bar, nonce),
            );
        }
    }
}
