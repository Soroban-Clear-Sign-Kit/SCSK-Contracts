#![no_std]
use soroban_sdk::auth::{
    Context, ContractContext, CustomAccountInterface, InvokerContractAuthEntry,
    SubContractInvocation,
};
use soroban_sdk::crypto::Hash;
use soroban_sdk::{
    Address, Env, IntoVal, String, Symbol, Val, Vec, contract, contracterror, contractevent,
    contractimpl, contracttype, symbol_short, token,
};

#[contractevent(data_format = "single-value")]
pub struct InnerEvent {
    #[topic]
    pub user: Address,
    pub value: i128,
}

/// Optional metadata attached to a forwarding operation.
#[contracttype]
pub struct Meta {
    /// A human-readable memo describing the transaction.
    pub memo: String,
    /// A list of tags for categorizing or indexing the operation.
    pub tags: Vec<Symbol>,
}

/// The execution mode for the forwarding operation.
#[contracttype]
pub enum Mode {
    /// Execute the forwarding immediately.
    Fast,
    /// Schedule the forwarding for a future timestamp.
    Scheduled(u64),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AccountError {
    NotAuthorized = 1,
}

/// A custom account contract implementing `CustomAccountInterface` (`__check_auth`).
#[contract]
pub struct CustomAccount;

#[contractimpl]
impl CustomAccountInterface for CustomAccount {
    type Signature = Val;
    type Error = AccountError;

    fn __check_auth(
        _env: Env,
        _signature_payload: Hash<32>,
        _signatures: Self::Signature,
        _auth_contexts: Vec<Context>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A fixture contract that forwards tokens and exercises authorization scenarios.
///
/// This contract is used for testing authorization flows, nested authorization trees,
/// contract-to-contract authorization, and custom user-defined types (UDTs).
#[contract]
pub struct Forwarder;

#[contractimpl]
impl Forwarder {
    /// Forwards tokens from one address to another using standard `require_auth`.
    pub fn forward(env: Env, from: Address, token: Address, to: Address, amount: i128) {
        do_transfer(&env, &from, &token, &to, &amount);
    }

    /// Forwards tokens attaching complex metadata and mode types.
    pub fn forward_with_meta(
        env: Env,
        from: Address,
        token: Address,
        to: Address,
        amount: i128,
        _meta: Meta,
        _mode: Mode,
    ) {
        do_transfer(&env, &from, &token, &to, &amount);
    }

    /// Forwards tokens using custom authorization arguments (`require_auth_for_args`).
    ///
    /// Demonstrates decoupling the top-level function arguments from the exact
    /// parameters authorized by the principal.
    pub fn forward_for_args(env: Env, from: Address, token: Address, to: Address, amount: i128) {
        from.require_auth_for_args(
            (
                symbol_short!("forw_args"),
                token.clone(),
                to.clone(),
                amount,
            )
                .into_val(&env),
        );
        token::Client::new(&env, &token).transfer(&from, &to, &amount);
    }

    /// Initiates a multi-contract call chain reaching depth 3+.
    ///
    /// Level 1: `chain_call` (requires `from` auth)
    /// Level 2: `chain_step` on `relay` (requires `from` auth)
    /// Level 3: `transfer` on `token` (requires `from` auth)
    pub fn chain_call(
        env: Env,
        from: Address,
        relay: Address,
        token: Address,
        to: Address,
        amount: i128,
    ) {
        from.require_auth();
        ForwarderClient::new(&env, &relay).chain_step(&from, &token, &to, &amount);
    }

    /// Intermediate step in a multi-contract call chain.
    pub fn chain_step(env: Env, from: Address, token: Address, to: Address, amount: i128) {
        from.require_auth();
        token::Client::new(&env, &token).transfer(&from, &to, &amount);
    }

    /// Exercises the same address authorizing at two distinct tree depths (Depth 1 and Depth 2).
    pub fn dual_depth_auth(env: Env, user: Address, sub_contract: Address, value: i128) {
        user.require_auth();
        ForwarderClient::new(&env, &sub_contract).inner_action(&user, &value);
    }

    /// Sub-invocation target that requires authorization from the same user at Depth 2.
    pub fn inner_action(env: Env, user: Address, value: i128) {
        user.require_auth();
        InnerEvent { user, value }.publish(&env);
    }

    /// Authorizes contract-to-contract calls on behalf of the current contract
    /// using `env.authorize_as_current_contract`.
    pub fn contract_transfer(
        env: Env,
        caller: Address,
        relay: Address,
        token: Address,
        to: Address,
        amount: i128,
    ) {
        caller.require_auth();
        let current = env.current_contract_address();
        env.authorize_as_current_contract(soroban_sdk::vec![
            &env,
            InvokerContractAuthEntry::Contract(SubContractInvocation {
                context: ContractContext {
                    contract: token.clone(),
                    fn_name: symbol_short!("transfer"),
                    args: (current.clone(), to.clone(), amount).into_val(&env),
                },
                sub_invocations: soroban_sdk::vec![&env],
            }),
        ]);
        ForwarderClient::new(&env, &relay).execute_transfer(&current, &token, &to, &amount);
    }

    /// Executes a delegated transfer requiring authorization from the `spender` contract.
    pub fn execute_transfer(env: Env, spender: Address, token: Address, to: Address, amount: i128) {
        spender.require_auth();
        token::Client::new(&env, &token).transfer(&spender, &to, &amount);
    }
}

/// Helper function to perform token transfer with standard authorization.
fn do_transfer(env: &Env, from: &Address, token: &Address, to: &Address, amount: &i128) {
    from.require_auth();
    token::Client::new(env, token).transfer(from, to, amount);
}

mod test;
