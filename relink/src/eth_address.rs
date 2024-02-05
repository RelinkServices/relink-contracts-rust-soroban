use core::fmt::{Display, Formatter};

use hex_slice::AsHex;
use soroban_sdk::unwrap::UnwrapOptimized;
use soroban_sdk::{contracttype, BytesN, Env};

#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
#[contracttype]
pub struct EthAddress(BytesN<20>);

impl EthAddress {
    /// Generates the ethereum address corresponding to the given sec-1 encoded secp256k1 public key.
    pub fn from_sec1_pub_key(env: &Env, pub_key: &BytesN<65>) -> Self {
        // drop the type-byte of sec-1 encoding
        let raw_pub_key = &pub_key.as_ref().slice(1..);
        // hash the raw public key
        let hash = env.crypto().keccak256(&raw_pub_key);
        // use the last 20 bytes as the address
        EthAddress(hash.as_ref().slice(12..).try_into().unwrap_optimized())
    }

    pub fn zero(env: &Env) -> Self {
        EthAddress(BytesN::from_array(env, &[0; 20]))
    }

    pub fn from_bytes(bytes: BytesN<20>) -> Self {
        EthAddress(bytes)
    }

    pub fn as_bytes(&self) -> &BytesN<20> {
        &self.0
    }
}

impl Display for EthAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "0x{:02x}", self.0.to_array().plain_hex(false))
    }
}

#[cfg(test)]
mod test {
    extern crate alloc;

    use alloc::format;

    use soroban_sdk::{bytesn, Env};

    use crate::EthAddress;

    #[test]
    fn generation() {
        let env = Env::default();
        let pub_key = bytesn!(&env, 0x0108ef37abca9f70c11265363057a0277eb61809ed1cd344c4db6134fba816a8fb21a09af9b592f08d0fa337f84c118a1d177f77e42d4b55fb806fe73ff7668c58);
        let address = EthAddress::from_sec1_pub_key(&env, &pub_key);
        assert_eq!(
            format!("{address}"),
            "0x67835910d32600471f388a137bbff3eb07993c04"
        );
    }
}
