use soroban_sdk::{bytes, contracttype, Address, Bytes, BytesN, Env, Vec};

use crate::utils::address_bytes;
use crate::{events, Error, EthAddress, RequestId, VrfDirectFundingProxyClient};

#[derive(Clone)]
#[contracttype]
pub enum DataKeyConsumer {
    DomainSeparator,
    Proxy,
    Threshold,
    Oracle(EthAddress),
}

pub fn init(env: &Env, proxy: &Address, threshold: u32, oracles: Vec<EthAddress>) {
    if has_proxy(&env) {
        panic!("already initialized")
    }
    // precalculate and store the domain separator
    set_domain_separator(env, &domain_separator(env, &env.current_contract_address()));
    set_proxy(env, proxy);
    set_threshold(env, threshold);
    for oracle in oracles {
        add_oracle(env, oracle);
    }
}

pub fn set_domain_separator(env: &Env, domain_separator: &BytesN<32>) {
    let key = DataKeyConsumer::DomainSeparator;
    env.storage().instance().set(&key, domain_separator);
}

pub fn get_domain_separator(env: &Env) -> BytesN<32> {
    let key = DataKeyConsumer::DomainSeparator;
    env.storage().instance().get(&key).unwrap()
}

pub fn set_proxy(env: &Env, proxy: &Address) {
    let key = DataKeyConsumer::Proxy;
    env.storage().instance().set(&key, proxy);
}

pub fn get_proxy(env: &Env) -> Address {
    let key = DataKeyConsumer::Proxy;
    env.storage().instance().get(&key).unwrap()
}

pub fn has_proxy(env: &Env) -> bool {
    let key = DataKeyConsumer::Proxy;
    env.storage().instance().has(&key)
}

pub fn set_threshold(env: &Env, threshold: u32) {
    env.storage()
        .instance()
        .set(&DataKeyConsumer::Threshold, &threshold);
    events::threshold_set(env, threshold);
}

pub fn get_threshold(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKeyConsumer::Threshold)
        .unwrap()
}

pub fn add_oracle(env: &Env, oracle: EthAddress) {
    env.storage()
        .instance()
        .set(&DataKeyConsumer::Oracle(oracle.clone()), &());
    events::oracle_added(env, oracle);
}

pub fn remove_oracle(env: &Env, oracle: EthAddress) {
    env.storage()
        .instance()
        .remove(&DataKeyConsumer::Oracle(oracle.clone()));
    events::oracle_removed(env, oracle);
}

pub fn has_oracle(env: &Env, oracle: EthAddress) -> bool {
    env.storage()
        .instance()
        .has(&DataKeyConsumer::Oracle(oracle))
}

/// Implement Solidity equivalent domain separator:
/// domainSeparator = keccak256(
///     abi.encode(
///         EIP712DOMAINTYPE_HASH,
///         NAME_HASH,
///         VERSION_HASH,
///         block.chainid,
///         address(this),
///         SALT
///     )
/// );
pub fn domain_separator(env: &Env, contract: &Address) -> BytesN<32> {
    // Keccak256 of "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)"
    let prefix_hash = bytes!(
        env,
        0xd87cd6ef79d4e2b95e15ce8abf732db51ec771f1ca2edccf22a46c729ac56472
    );
    // Keccak256 of "Relink MultiSig"
    let name_hash = bytes!(
        env,
        0x615770db9463494f4a7a0575fdd7fbbbdd2ac99f24e8a7960d5bd346cfa1dbf7
    );
    // Keccak256 of "1"
    let version_hash = bytes!(
        env,
        0xc89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6
    );
    let salt = bytes!(
        env,
        0x151543af6b722378665a73fe38dbceae4871a070b7cdaf5c6e30cf758dc33cc8
    );
    let mut domain = Bytes::new(env);
    domain.append(&prefix_hash);
    domain.append(&name_hash);
    domain.append(&version_hash);
    domain.append(env.ledger().network_id().as_ref());
    domain.append(&address_bytes(env, &contract));
    domain.append(&salt);
    env.crypto().keccak256(&domain)
}

/// Implement Solidity equivalent tx input hash:
/// bytes32 txInputHash = keccak256(
///     abi.encode(
///         RANDOMNESS_RECEIVED_HASH,
///         requestId,
///         keccak256(abi.encodePacked(randomWords))
///     )
/// );
pub fn tx_input_hash(env: &Env, id: &RequestId, random_words: &Vec<BytesN<32>>) -> BytesN<32> {
    // Keccak256 of "ProxyRequest(bytes32 requestId,uint256[] randomWords)"
    let prefix_hash = bytes!(
        env,
        0x910684821c06c34275feee35f4ae2d80136d3b2e7366d0795b2eb4116d82ac07
    );
    let crypto = env.crypto();
    // hash the random words
    let mut buffer = Bytes::new(env);
    for word in random_words.iter() {
        buffer.append(word.as_ref());
    }
    let random_words_hash = crypto.keccak256(&buffer);
    // hash the tx inputs
    let mut buffer = Bytes::new(env);
    buffer.append(&prefix_hash);
    buffer.append(id.as_bytes().as_ref());
    buffer.append(random_words_hash.as_ref());
    crypto.keccak256(&buffer)
}

pub fn request_randomness(
    env: &Env,
    origin: Address,
    value: i128,
    request_confirmations: Option<u32>,
    num_words: Option<u32>,
) -> RequestId {
    // require authorization by origin
    origin.require_auth();
    // call to the proxy contract
    let proxy_client = VrfDirectFundingProxyClient::new(env, &get_proxy(&env));
    proxy_client.request_randomness(
        &origin,
        &value,
        &env.current_contract_address(),
        &request_confirmations.unwrap_or(3),
        &num_words.unwrap_or(1),
    )
}

pub fn verify_randomness(
    env: &Env,
    id: &RequestId,
    random_words: &Vec<BytesN<32>>,
    signatures: &Vec<(BytesN<64>, u32)>,
) -> Result<(), Error> {
    // must be called by the proxy
    get_proxy(env).require_auth();
    // verify oracle signatures
    let threshold = get_threshold(env);
    if signatures.len() < threshold {
        return Err(Error::TooFewSignatures);
    }
    // reproduce message-to-sign
    // bytes32 totalHash = keccak256(
    //     abi.encodePacked("\x19\x01", domainSeparator, txInputHash)
    // );
    let mut msg = Bytes::from_array(env, &[0x19, 0x01]);
    msg.append(get_domain_separator(env).as_ref());
    msg.append(tx_input_hash(env, id, random_words).as_ref());
    let crypto = env.crypto();
    let msg_digest = crypto.keccak256(&msg);
    // validate oracle signatures, recovering the addresses in the process, this will panic on invalid signatures
    let mut valid: u32 = 0;
    let mut last = EthAddress::zero(env);
    for (signature, recovery_id) in signatures.iter() {
        let pub_key = crypto.secp256k1_recover(&msg_digest, &signature, recovery_id);
        // convert pub key to ethereum address (hash the raw pub key and take the last 20 bytes)
        let recovered = EthAddress::from_sec1_pub_key(env, &pub_key);
        // if the same address is used multiple times, fail
        if recovered <= last {
            return Err(Error::UnorderedOracles);
        }
        // check if the computed address is included in the permitted oracles list
        if has_oracle(env, recovered.clone()) {
            // address is a part of the permitted oracles list, increase verified signature counter
            valid += 1;
            // count valid signatures
            last = recovered;
        }
    }
    if valid < threshold {
        return Err(Error::UnauthorizedOracleSignatures);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    extern crate alloc;

    use soroban_sdk::testutils::arbitrary::std::println;
    use soroban_sdk::testutils::Address as AddressTestTrait;
    use soroban_sdk::{Address, BytesN, Env};

    use crate::consumer::domain_separator;
    use crate::testutils::{hash_hex, hash_u8};

    /// Precompute values used in domain_separator().
    #[test]
    fn precompute_hashes() {
        let env = Env::default();
        let domain_prefix = b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)";
        println!("EIP712DOMAINTYPE_HASH: {}", hash_u8(&env, domain_prefix));
        println!("NAME_HASH: {}", hash_u8(&env, b"Relink MultiSig"));
        println!("VERSION_HASH: {}", hash_u8(&env, b"1"));
        let signature_prefix = b"ProxyRequest(bytes32 requestId,uint256[] randomWords)";
        println!(
            "RANDOMNESS_RECEIVED_HASH: {}",
            hash_u8(&env, signature_prefix)
        );
        println!(
            "Network ID: Futurenet = {}",
            hash_u8(&env, b"Test SDF Future Network ; October 2022")
        );
        println!(
            "Network ID: Testnet = {}",
            hash_u8(&env, b"Test SDF Network ; September 2015")
        );
    }

    /// Compute the domain separator with the given contract address and verify that it matches the
    /// expected value in hex.
    fn verify_domain_separator(address: &Address, expected_hex: &str) -> BytesN<32> {
        let result = domain_separator(address.env(), address);
        println!("domain separator: {}", hash_hex(&result));
        assert_eq!(hash_hex(&result), expected_hex);
        result
    }

    #[test]
    fn verify_domain_separators() {
        let env = Env::default();
        let a = verify_domain_separator(
            &Address::generate(&env),
            "0x946a7d98c84e7d51a25c369f37e21091cef346c610a3eb9daeb942efc9012dc1",
        );
        let b = verify_domain_separator(
            &Address::generate(&env),
            "0x56aa7c55e371e1c518f8cc4ffad1d25155aa8c30cf88d8e9a41bf5a219fbc9ea",
        );
        // make sure that different contract addresses yield different domain separators
        assert_ne!(a, b);
    }
}
