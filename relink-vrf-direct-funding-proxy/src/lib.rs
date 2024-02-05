#![no_std]

use soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env, Vec};

use relink::{
    confirmed_owner, impl_confirmed_owner, ConfirmedOwner, Error, RequestId,
    VrfDirectFundingConsumerClient, VrfDirectFundingProxy,
};

pub mod events;
mod storage;
mod test;

#[contract]
pub struct RelinkVrfDirectFundingProxy;

impl_confirmed_owner!(RelinkVrfDirectFundingProxy);

#[contractimpl]
impl RelinkVrfDirectFundingProxy {
    /// Initialize contract by setting the owner address and token to use.
    pub fn initialize(env: Env, owner: Address, token: Address, fee: i128) {
        confirmed_owner::init(&env, &owner);
        storage::set_token(&env, &token);
        storage::set_fee(&env, &fee);
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

    /// Update the fee for calling request_randomness.
    pub fn set_fee(env: Env, fee: i128) {
        confirmed_owner::require_owner(&env);
        storage::set_fee(&env, &fee);
        events::fee_set(&env, fee);
    }

    /// Get the current fee for calling request_randomness.
    pub fn get_fee(env: Env) -> i128 {
        storage::get_fee(&env)
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
        origin: Address,
        value: i128,
        dapp: Address,
        request_confirmations: u32,
        num_words: u32,
    ) -> Result<RequestId, Error> {
        // assert a maxmium of 10 words
        if num_words > 10 {
            panic!("Maximum Random Values: 10");
        }
        // assert minimum fee
        if value < storage::get_fee(&env) {
            return Err(Error::InsufficientFee);
        }
        // require authorization by sender
        origin.require_auth();

        // transfer fee to this contract
        let token_id = storage::get_token(&env);
        let client = token::Client::new(&env, &token_id);
        client.transfer(&origin, &env.current_contract_address(), &value);

        // generate unique request id
        let nonce = storage::get_and_increment_nonce(&env);
        let id = RequestId::new(&env, &origin, &dapp, &env.current_contract_address(), nonce);

        // store a mapping of request-id => dapp address
        storage::set_request_dapp(&env, id.clone(), &dapp);

        // emit event
        events::randomness_requested(
            &env,
            origin,
            dapp,
            nonce,
            id.clone(),
            request_confirmations,
            num_words,
        );

        Ok(id)
    }

    /// Entry point for randomness data coming from a backend.
    fn callback_with_randomness(
        env: Env,
        backend: Address,
        id: RequestId,
        random_words: Vec<BytesN<32>>,
        signatures: Vec<(BytesN<64>, u32)>,
    ) -> Result<(), Error> {
        // only backends on the whitelist are allowed to call this
        backend.require_auth();
        storage::is_whitelisted(&env, backend)?;
        // read and then remove originating dapp address, error if not found
        let dapp = storage::get_request_dapp(&env, id.clone())?;
        storage::remove_request_dapp(&env, id.clone());
        // callback to dapp contract with provided random words
        let dapp_client = VrfDirectFundingConsumerClient::new(&env, &dapp);
        dapp_client.verify_and_fulfill_randomness(&id, &random_words, &signatures);
        // emit event containing the provided random words
        events::randomness_provided(&env, id, random_words);
        Ok(())
    }
}
