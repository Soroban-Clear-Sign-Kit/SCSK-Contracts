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

    /// Forwards the same `amount` from one address to every recipient in `tos`.
    ///
    /// Requires a single authorization from `from`. The authorization tree gets one
    /// nested `transfer` sub-invocation per recipient, which makes this useful for
    /// testing wide (rather than deep) authorization trees.
    pub fn forward_many(env: Env, from: Address, token: Address, tos: Vec<Address>, amount: i128) {
        from.require_auth();
        let client = token::Client::new(&env, &token);
        for to in tos.iter() {
            client.transfer(&from, &to, &amount);
        }
    }
}

/// Helper function to perform token transfer with standard authorization.
fn do_transfer(env: &Env, from: &Address, token: &Address, to: &Address, amount: &i128) {
    from.require_auth();
    token::Client::new(env, token).transfer(from, to, amount);
}

mod test;
