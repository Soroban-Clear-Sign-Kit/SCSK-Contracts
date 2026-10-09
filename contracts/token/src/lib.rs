#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, contractevent, Address, Env, String};

#[contractevent(data_format = "single-value")]
struct Transfer {
    #[topic]
    from: Address,
    #[topic]
    to: Address,
    amount: i128,
}

#[contractevent(data_format = "single-value")]
struct Approve {
    #[topic]
    from: Address,
    #[topic]
    spender: Address,
    data: (i128, u32),
}

#[contractevent(data_format = "single-value")]
struct Mint {
    #[topic]
    to: Address,
    amount: i128,
}

#[contractevent(data_format = "single-value")]
struct Burn {
    #[topic]
    from: Address,
    amount: i128,
}

#[contractevent(data_format = "single-value")]
struct Clawback {
    #[topic]
    from: Address,
    amount: i128,
}

#[contractevent(data_format = "single-value")]
struct SetAdmin {
    #[topic]
    admin: Address,
    new_admin: Address,
}

#[contractevent(data_format = "single-value")]
struct SetAuthorized {
    #[topic]
    admin: Address,
    #[topic]
    id: Address,
    authorize: bool,
}


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
        
        Transfer { from, to, amount }.publish(&env);
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
        
        Transfer { from, to, amount }.publish(&env);
    }

    pub fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();
        if amount < 0 {
            panic!("negative amount");
        }
        write_allowance(&env, &from, &spender, amount, expiration_ledger);
        Approve { from, spender, data: (amount, expiration_ledger) }.publish(&env);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        if amount < 0 {
            panic!("negative amount");
        }
        let to_bal = read_balance(&env, &to);
        write_balance(&env, &to, to_bal + amount);
        Mint { to, amount }.publish(&env);
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
        Burn { from, amount }.publish(&env);
    }

    pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
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
        Burn { from, amount }.publish(&env);
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
        Clawback { from, amount }.publish(&env);
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        SetAdmin { admin, new_admin }.publish(&env);
    }

    pub fn set_authorized(env: Env, id: Address, authorize: bool) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        env.storage().persistent().set(&DataKey::State(id.clone()), &authorize);
        SetAuthorized { admin, id, authorize }.publish(&env);
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
    extern crate std;
    use super::*;
    use soroban_sdk::{Env, testutils::{Address as _, Events}, IntoVal, Symbol};

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
        
        let all_events = env.events().all();
        assert_eq!(
            all_events,
            soroban_sdk::vec![
                &env,
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "mint"), user1.clone()).into_val(&env),
                    1000_i128.into_val(&env)
                )
            ]
        );
        
        assert_eq!(client.balance(&user1), 1000);
        
        // Transfer
        client.transfer(&user1, &user2, &200);
        
        assert_eq!(
            env.events().all(),
            soroban_sdk::vec![
                &env,
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "transfer"), user1.clone(), user2.clone()).into_val(&env),
                    200_i128.into_val(&env)
                )
            ]
        );
        
        assert_eq!(client.balance(&user1), 800);
        assert_eq!(client.balance(&user2), 200);

        // Approve
        client.approve(&user1, &user2, &500, &100);
        assert_eq!(
            env.events().all(),
            soroban_sdk::vec![
                &env,
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "approve"), user1.clone(), user2.clone()).into_val(&env),
                    (500_i128, 100_u32).into_val(&env)
                )
            ]
        );
        assert_eq!(client.allowance(&user1, &user2), 500);
        
        // Transfer from
        client.transfer_from(&user2, &user1, &user2, &300);
        assert_eq!(client.balance(&user1), 500);
        assert_eq!(client.balance(&user2), 500);
        assert_eq!(client.allowance(&user1, &user2), 200);

        // Burn
        client.burn(&user1, &100);
        assert_eq!(
            env.events().all(),
            soroban_sdk::vec![
                &env,
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "burn"), user1.clone()).into_val(&env),
                    100_i128.into_val(&env)
                )
            ]
        );
        assert_eq!(client.balance(&user1), 400);

        // Clawback
        client.clawback(&user2, &100);
        assert_eq!(
            env.events().all(),
            soroban_sdk::vec![
                &env,
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "clawback"), user2.clone()).into_val(&env),
                    100_i128.into_val(&env)
                )
            ]
        );
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

    #[test]
    #[should_panic]
    fn test_mint_without_admin_auth() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let contract_id = env.register(Token, ());
        let client = TokenClient::new(&env, &contract_id);
        client.initialize(&admin, &7, &String::from_str(&env, "Token"), &String::from_str(&env, "TKN"));
        
        // Not mocking auths, so this will panic when it requires admin auth
        client.mint(&user1, &100);
    }

    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_initialize_twice() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let contract_id = env.register(Token, ());
        let client = TokenClient::new(&env, &contract_id);
        client.initialize(&admin, &7, &String::from_str(&env, "Token"), &String::from_str(&env, "TKN"));
        client.initialize(&admin, &7, &String::from_str(&env, "Token"), &String::from_str(&env, "TKN"));
    }

    #[test]
    #[should_panic(expected = "insufficient allowance")]
    fn test_transfer_from_above_allowance() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let contract_id = env.register(Token, ());
        let client = TokenClient::new(&env, &contract_id);
        client.initialize(&admin, &7, &String::from_str(&env, "Token"), &String::from_str(&env, "TKN"));
        env.mock_all_auths();
        client.mint(&user1, &1000);
        client.approve(&user1, &user2, &100, &100);
        
        client.transfer_from(&user2, &user1, &user2, &101);
    }
}
