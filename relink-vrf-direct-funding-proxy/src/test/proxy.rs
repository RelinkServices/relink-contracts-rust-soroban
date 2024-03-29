extern crate std;

use std::println;

use soroban_sdk::{
    symbol_short,
    testutils::{Address as AddressTestTrait, Events},
    token, vec, Address, Env, IntoVal, Vec,
};

use relink::RequestId;

use crate::events::RandomnessRequestData;
use crate::{RelinkVrfDirectFundingProxy, RelinkVrfDirectFundingProxyClient};

struct Setup<'a> {
    env: Env,
    token: token::Client<'a>,
    token_admin: token::StellarAssetClient<'a>,
    token_owner: Address,
    proxy: RelinkVrfDirectFundingProxyClient<'a>,
    proxy_owner: Address,
}

impl Setup<'_> {
    fn new() -> Self {
        let env: Env = Env::default();
        env.mock_all_auths();

        // Create the token contract
        let token_owner = Address::generate(&env);
        let (token, token_admin) = relink::testutils::create_token_contract(&env, &token_owner);

        // Create the proxy contract
        let proxy_id = env.register_contract(None, RelinkVrfDirectFundingProxy);
        let proxy = RelinkVrfDirectFundingProxyClient::new(&env, &proxy_id);

        // initialize proxy contract
        let proxy_owner = Address::generate(&env);
        proxy.initialize(&proxy_owner, &token.address, &10);

        println!("token: {:?}", token.address);
        println!("token_owner: {:?}", token_owner);
        println!("proxy: {:?}", proxy.address);
        println!("proxy_owner: {:?}", proxy_owner);

        Self {
            env,
            token,
            token_admin,
            token_owner,
            proxy,
            proxy_owner,
        }
    }

    fn fund_account(&self, account: &Address, amount: &i128) {
        // Mint some tokens to work with
        self.token_admin.mint(account, amount);
    }

    fn random_account(&self, initial_amount: &i128) -> Address {
        let account = Address::generate(&self.env);
        self.fund_account(&account, initial_amount);
        account
    }
}

#[test]
fn init() {
    let setup = Setup::new();
    assert_eq!(setup.proxy.owner(), setup.proxy_owner);
    // assert_eq!(crate::storage::read_token(&setup.env), setup.token.address);
}

#[test]
#[should_panic(expected = "already initialized")]
fn reinit() {
    let setup = Setup::new();
    setup
        .proxy
        .initialize(&setup.proxy_owner, &setup.token.address, &123);
}

#[test]
fn request_ids() {
    let setup = Setup::new();
    let proxy = &setup.proxy;

    let origin = setup.random_account(&50);
    let dapp = setup.random_account(&0);
    let id1 = proxy.request_randomness(&origin, &10, &dapp, &2, &1);
    let id2 = proxy.request_randomness(&origin, &10, &dapp, &2, &1);
    let id3 = proxy.request_randomness(&origin, &10, &dapp, &2, &1);

    // make sure the request ids are not identical
    assert_ne!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn request_event() {
    let setup = Setup::new();
    let env = &setup.env;

    let origin = setup.random_account(&50);
    let dapp = setup.random_account(&0);
    let request_id = setup.proxy.request_randomness(&origin, &10, &dapp, &2, &1);
    println!("request id: {}", request_id);

    let mut proxy_events = Vec::new(env);

    // there are SAC events emitted also, filter those away, not asserting that aspect
    env.events()
        .all()
        .iter()
        .filter(|event| event.0 == setup.proxy.address)
        .for_each(|event| proxy_events.push_back(event));

    let expected_nonce = 0_u128;
    let expected_request_id =
        RequestId::new(env, &origin, &dapp, &setup.proxy.address, expected_nonce);

    assert_eq!(request_id, expected_request_id);

    assert_eq!(
        proxy_events,
        vec![
            env,
            (
                setup.proxy.address.clone(),
                (symbol_short!("request"), origin, dapp, expected_nonce).into_val(env),
                RandomnessRequestData {
                    id: expected_request_id,
                    request_confirmations: 2,
                    num_words: 1
                }
                .into_val(env),
            )
        ]
    );
}

#[test]
fn balance() {
    let setup = Setup::new();
    let token = &setup.token;
    let proxy = &setup.proxy;

    let origin1 = setup.random_account(&50);
    let origin2 = setup.random_account(&100);
    let dapp = setup.random_account(&0);

    assert_eq!(token.balance(&origin1), 50);
    assert_eq!(token.balance(&origin2), 100);
    assert_eq!(token.balance(&proxy.owner()), 0);
    assert_eq!(token.balance(&proxy.address), 0);

    proxy.request_randomness(&origin1, &10, &dapp, &2, &1);
    proxy.request_randomness(&origin2, &25, &dapp, &2, &5);
    proxy.request_randomness(&origin2, &11, &dapp, &2, &10);

    assert_eq!(token.balance(&origin1), 40);
    assert_eq!(token.balance(&origin2), 64);
    assert_eq!(token.balance(&proxy.owner()), 0);
    assert_eq!(token.balance(&proxy.address), 46);

    // withdraw all tokens to the owner
    proxy.withdraw(&token.address);

    assert_eq!(token.balance(&origin1), 40);
    assert_eq!(token.balance(&origin2), 64);
    assert_eq!(token.balance(&proxy.owner()), 46);
    assert_eq!(token.balance(&proxy.address), 0);
}

#[test]
#[should_panic(expected = "Maximum Random Values: 10")]
fn max_words() {
    let setup = Setup::new();
    let proxy = &setup.proxy;

    let origin = setup.random_account(&50);
    let dapp = setup.random_account(&0);

    proxy.request_randomness(&origin, &10, &dapp, &2, &11);
}
