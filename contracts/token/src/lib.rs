#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String};

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn transfer(_env: Env, from: Address, _to: Address, _amount: i128) {
        from.require_auth();
    }

    pub fn transfer_from(_env: Env, spender: Address, _from: Address, _to: Address, _amount: i128) {
        spender.require_auth();
    }

    pub fn approve(_env: Env, from: Address, _spender: Address, _amount: i128, _expiration_ledger: u32) {
        from.require_auth();
    }

    pub fn burn(_env: Env, from: Address, _amount: i128) {
        from.require_auth();
    }

    pub fn burn_from(_env: Env, spender: Address, _from: Address, _amount: i128) {
        spender.require_auth();
    }

    pub fn mint(_env: Env, _to: Address, _amount: i128) {
        // usually admin require_auth
    }

    pub fn clawback(_env: Env, _from: Address, _amount: i128) {
        // usually admin require_auth
    }

    pub fn set_admin(_env: Env, _new_admin: Address) {
        // usually admin require_auth
    }

    pub fn set_authorized(_env: Env, _id: Address, _authorize: bool) {
        // usually admin require_auth
    }

    pub fn balance(_env: Env, _id: Address) -> i128 {
        0
    }

    pub fn allowance(_env: Env, _from: Address, _spender: Address) -> i128 {
        0
    }

    pub fn decimals(_env: Env) -> u32 {
        7
    }

    pub fn name(env: Env) -> String {
        String::from_str(&env, "Test Token")
    }

    pub fn symbol(env: Env) -> String {
        String::from_str(&env, "TST")
    }
}
