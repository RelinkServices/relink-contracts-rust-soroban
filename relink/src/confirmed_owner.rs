use soroban_sdk::{contractclient, contracttype, Address, Env};

use crate::events;

#[contractclient(name = "ConfirmedOwnerClient")]
pub trait ConfirmedOwner {
    /// Return the current owner.
    fn owner(env: Env) -> Address;

    /// Initiate transfer of ownership. Must be authorized by the current owner.
    /// The owner will remain unchanged until the new owner has been confirmed.
    fn transfer_ownership(env: Env, to: Address);

    /// Accept and finalize transfer of ownership. Must be authorized by the new owner.
    fn accept_ownership(env: Env);
}

#[derive(Clone)]
#[contracttype]
enum DataKeyConfirmedOwner {
    Owner,
    PendingOwner,
}

fn has_owner(env: &Env) -> bool {
    let key = DataKeyConfirmedOwner::Owner;
    env.storage().instance().has(&key)
}

fn write_owner(env: &Env, address: &Address) {
    let key = DataKeyConfirmedOwner::Owner;
    env.storage().instance().set(&key, address);
}

fn read_owner(env: &Env) -> Address {
    let key = DataKeyConfirmedOwner::Owner;
    env.storage().instance().get(&key).unwrap()
}

fn write_pending_owner(env: &Env, address: &Address) {
    let key = DataKeyConfirmedOwner::PendingOwner;
    env.storage().instance().set(&key, address);
}

fn read_pending_owner(env: &Env) -> Address {
    let key = DataKeyConfirmedOwner::PendingOwner;
    env.storage().instance().get(&key).unwrap()
}

pub fn init(env: &Env, owner: &Address) {
    if has_owner(&env) {
        panic!("already initialized")
    }
    write_owner(env, owner);
}

/// Require authorization by the owner.
pub fn require_owner(env: &Env) {
    read_owner(env).require_auth();
}

pub fn owner(env: &Env) -> Address {
    read_owner(env)
}

pub fn transfer_ownership(env: &Env, to: &Address) {
    require_owner(env);
    write_pending_owner(&env, &to);
    events::ownership_transfer_requested(env, &read_owner(env), to);
}

pub fn accept_ownership(env: &Env) {
    let previous_owner = read_owner(&env);
    let new_owner = read_pending_owner(&env);
    // only the pending owner can call this
    new_owner.require_auth();
    // update owner
    write_owner(&env, &new_owner);
    events::ownership_transferred(env, &previous_owner, &new_owner);
}

#[macro_export]
macro_rules! impl_confirmed_owner {
    ($type:ident) => {
        #[contractimpl]
        impl confirmed_owner::ConfirmedOwner for $type {
            /// Return the current owner.
            fn owner(env: Env) -> Address {
                confirmed_owner::owner(&env)
            }

            /// Initiate transfer of ownership. Must be authorized by the current owner.
            /// The owner will remain unchanged until the new owner has been confirmed.
            fn transfer_ownership(env: Env, to: Address) {
                confirmed_owner::transfer_ownership(&env, &to)
            }

            /// Accept and finalize transfer of ownership. Must be authorized by the new owner.
            fn accept_ownership(env: Env) {
                confirmed_owner::accept_ownership(&env)
            }
        }
    };
}

#[cfg(test)]
mod test {
    extern crate std;

    use soroban_sdk::testutils::{
        Address as AddressTestTrait, AuthorizedFunction, AuthorizedInvocation,
    };
    use soroban_sdk::{contract, contractimpl, IntoVal, Symbol, Vec};

    use crate::confirmed_owner;

    use super::*;

    #[contract]
    struct TestContract;

    #[contractimpl]
    impl TestContract {
        pub fn init(env: Env, owner: Address) {
            confirmed_owner::init(&env, &owner);
        }

        /// Only the owner can call this.
        pub fn only_owner(env: Env) {
            require_owner(&env);
        }

        /// Anyone can call this if providing a signature.
        pub fn signed_caller(_: Env, sender: Address) {
            sender.require_auth();
        }

        /// Anyone can call this without a signature.
        pub fn unsigned(_: Env) {}
    }

    impl_confirmed_owner!(TestContract);

    fn setup<'a>() -> (Env, Address, TestContractClient<'a>) {
        let env = Env::default();
        env.mock_all_auths();

        // Create the test contract
        let contract_address = env.register_contract(None, TestContract);
        let client = TestContractClient::new(&env, &contract_address);

        (env, contract_address, client)
    }

    #[test]
    fn init() {
        let (env, _, client) = setup();
        // initialize test contract
        let alice = Address::generate(&env);
        client.init(&alice);
        assert_eq!(client.owner(), alice);
    }

    #[test]
    #[should_panic(expected = "already initialized")]
    fn reinit() {
        let (env, _, client) = setup();
        // initialize contract
        let alice = Address::generate(&env);
        client.init(&alice);
        // initializing again should fail
        client.init(&alice);
    }

    #[test]
    fn authorization() {
        let (env, contract_address, client) = setup();
        // initialize contract
        let alice = Address::generate(&env);
        client.init(&alice);

        // call a non-privileged method
        client.unsigned();
        // verify that no signature was required
        assert_eq!(env.auths(), std::vec![]);

        // call a privileged method
        let bob = Address::generate(&env);
        client.signed_caller(&bob);
        // verify that the bob's signature was checked
        assert_eq!(
            env.auths(),
            std::vec![(
                // Address for which authorization check is performed
                bob.clone(),
                // Invocation tree that needs to be authorized
                AuthorizedInvocation {
                    // Function that is authorized. Can be a contract function or a host function that requires authorization.
                    function: AuthorizedFunction::Contract((
                        // Address of the called contract
                        contract_address.clone(),
                        // Name of the called function
                        Symbol::new(&env, "signed_caller"),
                        // Arguments used to call `signed_caller` (converted to the env-managed vector via `into_val`)
                        (bob.clone(),).into_val(&env),
                    )),
                    // The contract doesn't call any other contracts that require authorization,
                    sub_invocations: std::vec![]
                }
            )]
        );

        // call a privileged method
        client.only_owner();
        // verify that the owner's signature was checked
        assert_eq!(
            env.auths(),
            std::vec![(
                // Address for which authorization check is performed
                alice.clone(),
                // Invocation tree that needs to be authorized
                AuthorizedInvocation {
                    // Function that is authorized. Can be a contract function or a host function that requires authorization.
                    function: AuthorizedFunction::Contract((
                        // Address of the called contract
                        contract_address.clone(),
                        // Name of the called function
                        Symbol::new(&env, "only_owner"),
                        // Arguments used to call `transfer_ownership` (converted to the env-managed vector via `into_val`)
                        Vec::new(&env),
                    )),
                    // The contract doesn't call any other contracts that require authorization,
                    sub_invocations: std::vec![]
                }
            )]
        );
    }

    #[test]
    fn transfer() {
        let (env, contract_address, client) = setup();
        // initialize contract
        let alice = Address::generate(&env);
        client.init(&alice);
        // make sure alice is the current owner
        assert_eq!(client.owner(), alice);

        // transfer ownership to bob
        let bob = Address::generate(&env);
        client.transfer_ownership(&bob);
        // verify that the owner's signature was checked
        assert_eq!(
            env.auths(),
            std::vec![(
                // Address for which authorization check is performed
                alice.clone(),
                // Invocation tree that needs to be authorized
                AuthorizedInvocation {
                    // Function that is authorized. Can be a contract function or a host function that requires authorization.
                    function: AuthorizedFunction::Contract((
                        // Address of the called contract
                        contract_address.clone(),
                        // Name of the called function
                        Symbol::new(&env, "transfer_ownership"),
                        // Arguments used to call `transfer_ownership` (converted to the env-managed vector via `into_val`)
                        (bob.clone(),).into_val(&env),
                    )),
                    // The contract doesn't call any other contracts that require authorization,
                    sub_invocations: std::vec![]
                }
            )]
        );

        // accept ownership with bob
        client.accept_ownership();
        // verify that bob's signature was checked
        assert_eq!(
            env.auths(),
            std::vec![(
                // Address for which authorization check is performed
                bob.clone(),
                // Invocation tree that needs to be authorized
                AuthorizedInvocation {
                    // Function that is authorized. Can be a contract function or a host function that requires authorization.
                    function: AuthorizedFunction::Contract((
                        // Address of the called contract
                        contract_address.clone(),
                        // Name of the called function
                        Symbol::new(&env, "accept_ownership"),
                        // Arguments used to call `accept_ownership` (converted to the env-managed vector via `into_val`)
                        Vec::new(&env),
                    )),
                    // The contract doesn't call any other contracts that require authorization,
                    sub_invocations: std::vec![]
                }
            )]
        );

        // verify bob is the owner now
        assert_eq!(client.owner(), bob);
    }
}
