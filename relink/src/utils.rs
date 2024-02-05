use soroban_sdk::unwrap::UnwrapOptimized;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{Address, Bytes, BytesN, Env};

/// Get raw bytes of an address.
pub fn address_bytes(env: &Env, address: &Address) -> Bytes {
    address.to_xdr(env).slice(8..40)
}

/// Get raw bytes of an address.
pub fn address_bytesn(env: &Env, address: &Address) -> BytesN<32> {
    address_bytes(env, address).try_into().unwrap_optimized()
}

#[cfg(test)]
mod test {
    extern crate alloc;

    use soroban_sdk::testutils::arbitrary::std::println;
    use soroban_sdk::{bytesn, Address, Env};

    #[test]
    fn address_bytes() {
        let env = Env::default();
        let expected_bytes = bytesn!(
            &env,
            0xcb328551c18dc4f779d05a19d106b7a2ec55d0e0cab461629a5fd503607b0ce6
        );
        // convert bytes to Strkey
        let strkey = stellar_strkey::Contract(expected_bytes.to_array());
        println!("Address: {strkey}");
        // convert strkey to address
        let address = Address::from_string(&soroban_sdk::String::from_str(
            &env,
            strkey.to_string().as_str(),
        ));
        // convert address back to bytes
        let actual_bytes = crate::utils::address_bytesn(&env, &address);
        assert_eq!(expected_bytes, actual_bytes);
    }
}
