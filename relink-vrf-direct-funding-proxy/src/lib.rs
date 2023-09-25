#![no_std]

use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, Map, Vec};

use relink::{
    confirmed_owner, impl_confirmed_owner, ConfirmedOwner, Error, RequestId,
    VrfDirectFundingConsumerClient, VrfDirectFundingProxy,
};

mod events;
mod storage;
mod test;

const FEE: i128 = 10;

#[contract]
pub struct RelinkVrfDirectFundingProxy;

impl_confirmed_owner!(RelinkVrfDirectFundingProxy);

#[contractimpl]
impl RelinkVrfDirectFundingProxy {
    /// Initialize contract by setting the owner address and token to use.
    pub fn initialize(env: Env, owner: Address, token: Address) {
        confirmed_owner::init(&env, &owner);
        storage::set_token(&env, &token);
    }

    /// Add an address to the backend whitelist.
    pub fn add_backend_whitelist(env: Env, address: Address) {
        confirmed_owner::require_owner(&env);
        storage::add_whitelist(&env, address.clone());
        events::whitelist_address_added(&env, address.clone());
    }

    /// Remove an address from the backend whitelist.
    pub fn remove_backend_whitelist(env: Env, address: Address) {
        confirmed_owner::require_owner(&env);
        storage::remove_whitelist(&env, address.clone());
        events::whitelist_address_removed(&env, address.clone());
    }

    /// Withdraw all tokens from the contract to the owner.
    pub fn withdraw(env: Env, token: Address) {
        confirmed_owner::require_owner(&env);
        let client = token::Client::new(&env, &token);
        let this_contract = env.current_contract_address();
        // withdraw all tokens to the owner
        let balance = client.balance(&this_contract);
        let owner = confirmed_owner::owner(&env);
        client.transfer(&this_contract, &owner, &balance);
    }
}

#[contractimpl]
impl VrfDirectFundingProxy for RelinkVrfDirectFundingProxy {
    /// Generate a new request for randomness.
    fn request_randomness(
        env: Env,
        sender: Address,
        value: i128,
        dapp: Address,
        request_confirmations: u32,
        num_words: u32,
    ) -> Result<RequestId, Error> {
        // TODO: assert num_words < 10?
        // require authorization by sender
        sender.require_auth();

        // assert minimum fee
        if value < FEE {
            return Err(Error::InsufficientFee);
        }

        // transfer fee to this contract
        let token_id = storage::get_token(&env);
        let client = token::Client::new(&env, &token_id);
        client.transfer(&sender, &env.current_contract_address(), &value);

        // generate unique request id
        let nonce = storage::get_and_increment_nonce(&env);
        let request_id = RequestId::new(&env, &env.current_contract_address(), nonce);

        // store a mapping of request-id => originating address
        storage::set_request_origin(&env, request_id.clone(), &dapp);

        // emit event
        events::randomness_requested(
            &env,
            sender,
            dapp,
            nonce,
            request_id.clone(),
            request_confirmations,
            num_words,
        );

        Ok(request_id)
    }

    /// Entry point for randomness data coming from a backend.
    fn callback_with_randomness(
        env: Env,
        backend: Address,
        request_origin: Address,
        id: RequestId,
        random_words: Vec<BytesN<32>>,
        signatures: Map<BytesN<32>, BytesN<64>>,
    ) -> Result<(), Error> {
        // only backends on the whitelist are allowed to call this
        backend.require_auth();
        storage::is_whitelisted(&env, backend)?;
        // read and then remove origin address, error if not found
        let dapp = storage::get_request_origin(&env, id.clone())?;
        storage::remove_request_origin(&env, id.clone());
        // callback to origin contract with provided random words
        let client = VrfDirectFundingConsumerClient::new(&env, &dapp);
        client.verify_and_fulfill_randomness(&id, &request_origin, &random_words, &signatures);
        Ok(())
    }
}
