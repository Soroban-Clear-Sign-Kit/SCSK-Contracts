#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

#[contract]
pub struct FixtureContract;

#[contractimpl]
impl FixtureContract {
    pub fn simple_call(_env: Env, arg_int: i32, arg_str: String, arg_addr: Address) -> bool {
        // Just return true
        true
    }

    pub fn transfer_auth(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        // Emulate some transfer logic, maybe just emit an event
        env.events().publish((Symbol::new(&env, "transfer"), from, to), amount);
    }
    
    pub fn nested_auth(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth_for_args(
            (Symbol::new(&env, "transfer_auth"), from.clone(), to.clone(), amount).into_val(&env)
        );
    }
    
    pub fn mint(env: Env, to: Address, amount: i128) {
        env.events().publish((Symbol::new(&env, "mint"), to), amount);
    }
}
