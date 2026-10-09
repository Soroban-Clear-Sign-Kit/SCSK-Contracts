#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Allowance(AllowanceDataKey),
    Balance(Address),
    Nonce(Address),
    State(Address),
    Admin,
    Decimals,
    Name,
    Symbol,
}

#[contracttype]
#[derive(Clone)]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

fn read_balance(env: &Env, id: &Address) -> i128 {
    env.storage().persistent().get(&DataKey::Balance(id.clone())).unwrap_or(0)
}

fn write_balance(env: &Env, id: &Address, amount: i128) {
    env.storage().persistent().set(&DataKey::Balance(id.clone()), &amount);
}

fn read_allowance(env: &Env, from: &Address, spender: &Address) -> AllowanceValue {
    let key = DataKey::Allowance(AllowanceDataKey { from: from.clone(), spender: spender.clone() });
    env.storage().persistent().get(&key).unwrap_or(AllowanceValue { amount: 0, expiration_ledger: 0 })
}

fn write_allowance(env: &Env, from: &Address, spender: &Address, amount: i128, expiration_ledger: u32) {
    let key = DataKey::Allowance(AllowanceDataKey { from: from.clone(), spender: spender.clone() });
    env.storage().persistent().set(&key, &AllowanceValue { amount, expiration_ledger });
}

#[contract]
pub struct Token;

#[contractimpl]
impl Token {
    pub fn initialize(env: Env, admin: Address, decimals: u32, name: String, symbol: String) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Decimals, &decimals);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        if amount < 0 {
            panic!("negative amount");
        }
        let from_bal = read_balance(&env, &from);
        if from_bal < amount {
            panic!("insufficient balance");
        }
        write_balance(&env, &from, from_bal - amount);
        let to_bal = read_balance(&env, &to);
        write_balance(&env, &to, to_bal + amount);
        
        env.events().publish((Symbol::new(&env, "transfer"), from, to), amount);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        if amount < 0 {
            panic!("negative amount");
        }
        let mut allowance = read_allowance(&env, &from, &spender);
        if allowance.amount < amount {
            panic!("insufficient allowance");
        }
        allowance.amount -= amount;
        write_allowance(&env, &from, &spender, allowance.amount, allowance.expiration_ledger);
        
        let from_bal = read_balance(&env, &from);
        if from_bal < amount {
            panic!("insufficient balance");
        }
        write_balance(&env, &from, from_bal - amount);
        let to_bal = read_balance(&env, &to);
        write_balance(&env, &to, to_bal + amount);
        
        env.events().publish((Symbol::new(&env, "transfer"), from, to), amount);
    }

    pub fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();
        if amount < 0 {
            panic!("negative amount");
        }
        write_allowance(&env, &from, &spender, amount, expiration_ledger);
        env.events().publish((Symbol::new(&env, "approve"), from, spender), (amount, expiration_ledger));
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        if amount < 0 {
            panic!("negative amount");
        }
        let to_bal = read_balance(&env, &to);
        write_balance(&env, &to, to_bal + amount);
        env.events().publish((Symbol::new(&env, "mint"), to), amount);
    }

    pub fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();
        if amount < 0 {
            panic!("negative amount");
        }
        let from_bal = read_balance(&env, &from);
        if from_bal < amount {
            panic!("insufficient balance");
        }
        write_balance(&env, &from, from_bal - amount);
        env.events().publish((Symbol::new(&env, "burn"), from), amount);
    }

    pub fn clawback(env: Env, from: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        if amount < 0 {
            panic!("negative amount");
        }
        let from_bal = read_balance(&env, &from);
        if from_bal < amount {
            panic!("insufficient balance");
        }
        write_balance(&env, &from, from_bal - amount);
        env.events().publish((Symbol::new(&env, "clawback"), from), amount);
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        env.events().publish((Symbol::new(&env, "set_admin"), admin), new_admin);
    }

    pub fn set_authorized(env: Env, id: Address, authorize: bool) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().persistent().set(&DataKey::State(id.clone()), &authorize);
        env.events().publish((Symbol::new(&env, "set_authorized"), admin, id), authorize);
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        read_balance(&env, &id)
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        read_allowance(&env, &from, &spender).amount
    }

    pub fn decimals(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::Decimals).unwrap_or(7)
    }

    pub fn name(env: Env) -> String {
        env.storage().instance().get(&DataKey::Name).unwrap_or_else(|| String::from_str(&env, "Test Token"))
    }

    pub fn symbol(env: Env) -> String {
        env.storage().instance().get(&DataKey::Symbol).unwrap_or_else(|| String::from_str(&env, "TST"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Env, testutils::{Address as _, Events}, vec, IntoVal};

    #[test]
    fn test_token_flow() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        
        let contract_id = env.register(Token, ());
        let client = TokenClient::new(&env, &contract_id);
        
        client.initialize(&admin, &7, &String::from_str(&env, "Token"), &String::from_str(&env, "TKN"));

        // Mint
        env.mock_all_auths();
        client.mint(&user1, &1000);
        assert_eq!(client.balance(&user1), 1000);
        
        // Transfer
        client.transfer(&user1, &user2, &200);
        assert_eq!(client.balance(&user1), 800);
        assert_eq!(client.balance(&user2), 200);

        // Approve and transfer_from
        client.approve(&user1, &user2, &500, &100);
        assert_eq!(client.allowance(&user1, &user2), 500);
        
        client.transfer_from(&user2, &user1, &user2, &300);
        assert_eq!(client.balance(&user1), 500);
        assert_eq!(client.balance(&user2), 500);
        assert_eq!(client.allowance(&user1, &user2), 200);
        
        // Burn
        client.burn(&user1, &100);
        assert_eq!(client.balance(&user1), 400);

        // Clawback
        client.clawback(&user2, &100);
        assert_eq!(client.balance(&user2), 400);
    }
    
    #[test]
    #[should_panic(expected = "insufficient balance")]
    fn test_transfer_above_balance() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let contract_id = env.register(Token, ());
        let client = TokenClient::new(&env, &contract_id);
        client.initialize(&admin, &7, &String::from_str(&env, "Token"), &String::from_str(&env, "TKN"));
        env.mock_all_auths();
        client.mint(&user1, &100);
        client.transfer(&user1, &user2, &101);
    }
}
