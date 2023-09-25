use core::fmt::{Display, Formatter};

use hex_slice::AsHex;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[contracttype]
pub struct RequestId(BytesN<32>);

impl RequestId {
    pub fn new(env: &Env, contract: &Address, nonce: u128) -> Self {
        let mut bytes = Bytes::new(env);
        bytes.append(&contract.to_xdr(env));
        bytes.append(&nonce.to_xdr(env));
        RequestId(env.crypto().sha256(&bytes))
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

    use soroban_sdk::testutils::Address as AddressTestTrait;
    use soroban_sdk::{Address, BytesN, Env};

    use crate::RequestId;

    #[test]
    fn hex() {
        let env = Env::default();
        let address = Address::from_contract_id(&BytesN::from_array(&env, &[0u8; 32]));
        let id = RequestId::new(&env, &address, 0);
        assert_eq!(
            format!("{}", id),
            "0x5cb9e7cf2ca4df23269cc87c879bff59b155e847e2fe6240146cd0855cb74ede"
        );
    }

    #[test]
    fn empty() {
        let env = Env::default();
        let address = Address::from_contract_id(&BytesN::from_array(&env, &[0u8; 32]));
        let id = RequestId::new(&env, &address, 0);
        assert_eq!(
            id,
            RequestId::from_bytes(BytesN::from_array(
                &env,
                &[
                    92, 185, 231, 207, 44, 164, 223, 35, 38, 156, 200, 124, 135, 155, 255, 89, 177,
                    85, 232, 71, 226, 254, 98, 64, 20, 108, 208, 133, 92, 183, 78, 222
                ]
            ))
        );
    }

    #[test]
    fn random() {
        let env = Env::default();
        let addr1 = Address::random(&env);
        let addr2 = Address::random(&env);
        for nonce in 0..100 {
            assert_eq!(
                RequestId::new(&env, &addr1, nonce),
                RequestId::new(&env, &addr1, nonce)
            );
            assert_ne!(
                RequestId::new(&env, &addr1, nonce),
                RequestId::new(&env, &addr1, nonce + 1)
            );
            assert_ne!(
                RequestId::new(&env, &addr1, nonce),
                RequestId::new(&env, &addr2, nonce)
            );
        }
    }
}
