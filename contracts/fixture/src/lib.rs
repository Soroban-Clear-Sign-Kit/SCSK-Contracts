#![no_std]
use soroban_sdk::{Address, Env, String, Symbol, Vec, contract, contractimpl, contracttype, token};

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

/// A fixture contract that forwards tokens between accounts.
///
/// This contract is used primarily for testing authorization flows,
/// nested contract calls, and complex parameter parsing in the Clear Sign Kit backend.
#[contract]
pub struct Forwarder;

#[contractimpl]
impl Forwarder {
    /// Forwards tokens from one address to another.
    ///
    /// Requires authorization from the `from` address.
    ///
    /// # Arguments
    /// * `env` - The environment.
    /// * `from` - The account sending the tokens.
    /// * `token` - The contract ID of the token to transfer.
    /// * `to` - The account receiving the tokens.
    /// * `amount` - The amount of tokens to transfer.
    pub fn forward(env: Env, from: Address, token: Address, to: Address, amount: i128) {
        do_transfer(&env, &from, &token, &to, &amount);
    }

    /// Forwards tokens from one address to another, attaching metadata and a mode.
    ///
    /// Similar to `forward`, but accepts additional complex types (`Meta` and `Mode`)
    /// to test the backend's ability to decode and format custom user-defined types (UDTs).
    ///
    /// Requires authorization from the `from` address.
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
}

/// Helper function to perform the token transfer and enforce authorization.
fn do_transfer(env: &Env, from: &Address, token: &Address, to: &Address, amount: &i128) {
    from.require_auth();
    token::Client::new(env, token).transfer(from, to, amount);
}

mod test;
