#![no_std]
use soroban_sdk::{Address, Env, String, Symbol, Vec, contract, contractimpl, contracttype, token};

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
        do_transfer(&env, &from, &token, &to, &amount);
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
        do_transfer(&env, &from, &token, &to, &amount);
    }
}

fn do_transfer(env: &Env, from: &Address, token: &Address, to: &Address, amount: &i128) {
    from.require_auth();
    token::Client::new(env, token).transfer(from, to, amount);
}

mod test;
