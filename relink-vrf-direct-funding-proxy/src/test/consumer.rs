extern crate std;

use std::println;

use relink::testutils::TestOracleGenerator;
use soroban_sdk::testutils::{
    Address as AddressTestTrait, AuthorizedFunction, AuthorizedInvocation,
};
use soroban_sdk::{token, vec, Address, BytesN, Env, IntoVal, Symbol};

use crate::test::test_consumer::{TestConsumer, TestConsumerClient};
use crate::{RelinkVrfDirectFundingProxy, RelinkVrfDirectFundingProxyClient};

struct Setup<'a> {
    env: Env,
    token: token::Client<'a>,
    token_admin: token::StellarAssetClient<'a>,
    token_owner: Address,
    proxy: RelinkVrfDirectFundingProxyClient<'a>,
    consumer: TestConsumerClient<'a>,
}

impl Setup<'_> {
    fn new() -> Self {
        let env = Env::default();
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

        // Create the consumer contract
        let consumer_id = env.register_contract(None, TestConsumer);
        let consumer = TestConsumerClient::new(&env, &consumer_id);

        println!("token: {:?}", token.address);
        println!("token_owner: {:?}", token_owner);
        println!("proxy: {:?}", proxy.address);
        println!("proxy_owner: {:?}", proxy_owner);
        println!("consumer: {:?}", consumer.address);

        Self {
            env,
            token,
            token_admin,
            token_owner,
            proxy,
            consumer,
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
fn request_randomness() {
    let setup = Setup::new();
    let proxy = &setup.proxy;
    let consumer = &setup.consumer;
    let env = &setup.env;

    // generate some key pairs for oracles
    let oracles = TestOracleGenerator::new().generate_sorted(env, 3);
    let oracle1 = &oracles[0];
    let oracle2 = &oracles[1];
    let oracle3 = &oracles[2];

    // initialize consumer contract
    let oracles_pub_keys = vec![
        env,
        oracle1.address(env),
        oracle2.address(env),
        oracle3.address(env),
    ];
    println!("oracle1 address: {}", oracle1.address(&env));
    println!("oracle2 address: {}", oracle2.address(&env));
    println!("oracle3 address: {}", oracle3.address(&env));
    consumer.initialize(&proxy.address, &2, &oracles_pub_keys);

    // initiate some requests
    let user = setup.random_account(&50);
    let id1 = consumer.initiate_randomness_request(&user, &10);
    let id2 = consumer.initiate_randomness_request(&user, &10);
    let id3 = consumer.initiate_randomness_request(&user, &10);
    println!("request id1: {}", id1);
    println!("request id2: {}", id2);
    println!("request id3: {}", id3);

    // make sure the request ids are not identical
    assert_ne!(id1, id2);
    assert_ne!(id1, id3);

    // add a backend to the whitelist
    let backend = setup.random_account(&0);
    proxy.add_backend_whitelist(&backend);

    // produce some randomness
    let random_words = vec![
        env,
        BytesN::from_array(
            &env,
            &[
                0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3,
                4, 5, 6, 7,
            ],
        ),
    ];
    // generate oracle signatures
    let signatures = vec![
        env,
        oracle1.sign(&env, &consumer.address, &id1, &random_words),
        // oracle2.sign(&env, &consumer.address, &id1, &random_words),
        oracle3.sign(&env, &consumer.address, &id1, &random_words),
    ];

    // execute the callback with randomness
    proxy.callback_with_randomness(&backend, &id1, &random_words, &signatures);

    // verify that the backend's signature was checked
    assert_eq!(
        env.auths(),
        std::vec![(
            // Address for which authorization check is performed
            backend.clone(),
            // Invocation tree that needs to be authorized
            AuthorizedInvocation {
                // Function that is authorized. Can be a contract function or a host function that requires authorization.
                function: AuthorizedFunction::Contract((
                    // Address of the called contract
                    proxy.address.clone(),
                    // Name of the called function
                    Symbol::new(&env, "callback_with_randomness"),
                    // Arguments used to call `callback_with_randomness` (converted to the env-managed vector via `into_val`)
                    (backend, id1, random_words, signatures).into_val(env),
                )),
                // The contract doesn't call any other contracts that require authorization,
                sub_invocations: std::vec![]
            }
        )]
    );
}
