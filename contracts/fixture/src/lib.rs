#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec, token};

#[contracttype]
pub struct Meta {
    pub memo: String,
    pub tags: Vec<Symbol>,
}

#[contracttype]
pub enum Mode {
    Fast,
    Scheduled(u64),
}

#[contract]
pub struct Forwarder;

#[contractimpl]
impl Forwarder {
    pub fn forward(env: Env, from: Address, token: Address, to: Address, amount: i128) {
        from.require_auth();
        token::Client::new(&env, &token).transfer(&from, &to, &amount);
    }

    pub fn forward_with_meta(
        env: Env,
        from: Address,
        token: Address,
        to: Address,
        amount: i128,
        _meta: Meta,
        _mode: Mode,
    ) {
        from.require_auth();
        token::Client::new(&env, &token).transfer(&from, &to, &amount);
    }
}
