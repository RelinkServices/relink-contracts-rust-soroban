use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{bytes, contracttype, Address, Bytes, BytesN, Env, Map, Vec};

use crate::{events, Error, RequestId, VrfDirectFundingProxyClient};

#[derive(Clone)]
#[contracttype]
pub enum DataKeyConsumer {
    DomainSeparator,
    Proxy,
    Threshold,
    Oracle(BytesN<32>),
}

pub fn init(env: &Env, proxy: &Address, threshold: u32, oracles: Vec<BytesN<32>>) {
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

pub fn add_oracle(env: &Env, pub_key: BytesN<32>) {
    env.storage()
        .instance()
        .set(&DataKeyConsumer::Oracle(pub_key.clone()), &());
    events::oracle_added(env, pub_key);
}

pub fn remove_oracle(env: &Env, pub_key: BytesN<32>) {
    env.storage()
        .instance()
        .remove(&DataKeyConsumer::Oracle(pub_key.clone()));
    events::oracle_removed(env, pub_key);
}

pub fn has_oracle(env: &Env, pub_key: BytesN<32>) -> bool {
    env.storage()
        .instance()
        .has(&DataKeyConsumer::Oracle(pub_key))
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
    // SHA256 of "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)"
    let prefix_hash = bytes!(
        env,
        0xad6c7f80cec084c59e505f24c9b5948b5dacc8024a22c878de1f6834e81414b6
    );
    // SHA256 of "Relink MultiSig"
    let name_hash = bytes!(
        env,
        0xa31ec8c002de82e32efa9ce1f5ec1e3d5109aacaae80ce0ee2bf905157da92f7
    );
    // SHA256 of "1"
    let version_hash = bytes!(
        env,
        0x6b86b273ff34fce19d6b804eff5a3f5747ada4eaa22f1d49c01e52ddb7875b4b
    );
    let salt = bytes!(
        env,
        0x151543af6b722378665a73fe38dbceae4871a070b7cdaf5c6e30cf758dc33cc8
    );
    let mut domain = Bytes::new(env);
    domain.append(&prefix_hash);
    domain.append(&name_hash);
    domain.append(&version_hash);
    domain.append(&env.ledger().network_id().to_xdr(env));
    domain.append(&contract.to_xdr(env));
    domain.append(&salt);
    env.crypto().sha256(&domain)
}

/// Implement Solidity equivalent tx input hash:
/// bytes32 txInputHash = keccak256(
///     abi.encode(
///         RANDOMNESS_RECEIVED_HASH,
///         requestOrigin,
///         chainId,
///         requestId,
///         keccak256(abi.encodePacked(randomWords))
///     )
/// );
pub fn tx_input_hash(
    env: &Env,
    id: &RequestId,
    request_origin: &Address,
    random_words: &Vec<BytesN<32>>,
) -> BytesN<32> {
    // SHA256 of "ProxyRequest(address requestOrigin,uint256 chainId,bytes32 requestId,uint256[] randomWords)"
    let prefix_hash = bytes!(
        env,
        0x55e5bc5c207cdaeab0f33b0afe4f990bb61a2d7d5e5691377ccb179eaf41463e
    );
    let mut tx_input = Bytes::new(env);
    tx_input.append(&prefix_hash);
    tx_input.append(&request_origin.to_xdr(env));
    tx_input.append(&env.ledger().network_id().to_xdr(env));
    tx_input.append(id.as_bytes().as_ref());
    tx_input.append(&random_words.clone().to_xdr(env));
    env.crypto().sha256(&tx_input)
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
    request_origin: &Address,
    random_words: &Vec<BytesN<32>>,
    signatures: &Map<BytesN<32>, BytesN<64>>,
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
    let mut msg = Bytes::new(env);
    // TODO: do we also need a prefix like \x19\x01 on Ethereum
    msg.append(get_domain_separator(env).as_ref());
    msg.append(tx_input_hash(env, id, request_origin, random_words).as_ref());
    let crypto = env.crypto();
    // note: signatures is a Map with the pub_keys as keys,
    // which guarantees that only one signature is provided per oracle
    let valid = signatures
        .iter()
        // keep only permitted oracles
        .filter(|(pub_key, _)| has_oracle(env, pub_key.clone()))
        // validate oracle signatures, this will panic on invalid signatures
        .map(|(pub_key, signature)| crypto.ed25519_verify(&pub_key, &msg, &signature))
        .count();
    if valid < threshold as usize {
        return Err(Error::UnauthorizedOracleSignatures);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    extern crate alloc;

    use crate::consumer::domain_separator;
    use crate::testutils::{hash_hex, hash_u8};
    use soroban_sdk::arbitrary::std::println;
    use soroban_sdk::{bytesn, Address, Env};

    /// Precompute values used in domain_separator().
    #[test]
    pub fn precompute_hashes() {
        let env = Env::default();
        let domain_prefix = b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)";
        println!("EIP712DOMAINTYPE_HASH: {}", hash_u8(&env, domain_prefix));
        println!("NAME_HASH: {}", hash_u8(&env, b"Relink MultiSig"));
        println!("VERSION_HASH: {}", hash_u8(&env, b"1"));
        let signature_prefix = b"ProxyRequest(address requestOrigin,uint256 chainId,bytes32 requestId,uint256[] randomWords)";
        println!(
            "RANDOMNESS_RECEIVED_HASH: {}",
            hash_u8(&env, signature_prefix)
        );
    }

    #[test]
    pub fn verify_domain_separator() {
        let env = Env::default();
        let contract1 = Address::from_contract_id(&bytesn!(
            &env,
            0x0102030405060708091011121314151617181920212223242526272829303132
        ));
        let contract2 = Address::from_contract_id(&bytesn!(
            &env,
            0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
        ));
        let result1 = domain_separator(&env, &contract1);
        let result2 = domain_separator(&env, &contract2);
        println!("domain separator 1: {}", hash_hex(&result1));
        println!("domain separator 2: {}", hash_hex(&result2));
        assert_ne!(result1, result2);
        assert_eq!(
            hash_hex(&result1),
            "0x35aa8170289291943109a27eecc1286b9875b1551c9adfd5a9489a273024bc03"
        );
        assert_eq!(
            hash_hex(&result2),
            "0x2840b5a1b6210913c6f8e48daf0b8c712e363a8d7c1ef23754adcaad2e32c032"
        );
    }
}
