#![no_std]
use soroban_sdk::{contract, contractevent, contractimpl, contracttype, Address, Env, String};

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

fn read_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("not initialized")
}

fn require_non_negative(amount: i128) {
    if amount < 0 {
        panic!("negative amount");
    }
}

fn read_balance(env: &Env, id: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(id.clone()))
        .unwrap_or(0)
}

fn write_balance(env: &Env, id: &Address, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::Balance(id.clone()), &amount);
}

fn read_authorized(env: &Env, id: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::State(id.clone()))
        .unwrap_or(true)
}

fn require_authorized(env: &Env, id: &Address) {
    if !read_authorized(env, id) {
        panic!("account is not authorized");
    }
}

fn spend_balance(env: &Env, id: &Address, amount: i128) {
    let balance = read_balance(env, id);
    if balance < amount {
        panic!("insufficient balance");
    }
    write_balance(env, id, balance - amount);
}

fn receive_balance(env: &Env, id: &Address, amount: i128) {
    let balance = read_balance(env, id);
    let new_balance = balance.checked_add(amount).expect("balance overflow");
    write_balance(env, id, new_balance);
}

fn read_allowance(env: &Env, from: &Address, spender: &Address) -> AllowanceValue {
    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });
    let allowance: AllowanceValue =
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(AllowanceValue {
                amount: 0,
                expiration_ledger: 0,
            });
    // An allowance whose expiration ledger has passed is treated as zero.
    if allowance.amount > 0 && allowance.expiration_ledger < env.ledger().sequence() {
        return AllowanceValue {
            amount: 0,
            expiration_ledger: allowance.expiration_ledger,
        };
    }
    allowance
}

fn write_allowance(
    env: &Env,
    from: &Address,
    spender: &Address,
    amount: i128,
    expiration_ledger: u32,
) {
    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });
    env.storage().persistent().set(
        &key,
        &AllowanceValue {
            amount,
            expiration_ledger,
        },
    );
}

fn spend_allowance(env: &Env, from: &Address, spender: &Address, amount: i128) {
    let allowance = read_allowance(env, from, spender);
    if allowance.amount < amount {
        panic!("insufficient allowance");
    }
    write_allowance(
        env,
        from,
        spender,
        allowance.amount - amount,
        allowance.expiration_ledger,
    );
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
        require_non_negative(amount);
        require_authorized(&env, &from);
        require_authorized(&env, &to);
        spend_balance(&env, &from, amount);
        receive_balance(&env, &to, amount);

        Transfer { from, to, amount }.publish(&env);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        require_non_negative(amount);
        require_authorized(&env, &from);
        require_authorized(&env, &to);
        spend_allowance(&env, &from, &spender, amount);

        spend_balance(&env, &from, amount);
        receive_balance(&env, &to, amount);

        Transfer { from, to, amount }.publish(&env);
    }

    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) {
        from.require_auth();
        require_non_negative(amount);
        if amount > 0 && expiration_ledger < env.ledger().sequence() {
            panic!("expiration ledger is in the past");
        }
        write_allowance(&env, &from, &spender, amount, expiration_ledger);
        Approve {
            from,
            spender,
            data: (amount, expiration_ledger),
        }
        .publish(&env);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin = read_admin(&env);
        admin.require_auth();
        require_non_negative(amount);
        require_authorized(&env, &to);
        receive_balance(&env, &to, amount);
        Mint { to, amount }.publish(&env);
    }

    pub fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();
        require_non_negative(amount);
        spend_balance(&env, &from, amount);
        Burn { from, amount }.publish(&env);
    }

    pub fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();
        require_non_negative(amount);
        spend_allowance(&env, &from, &spender, amount);

        spend_balance(&env, &from, amount);
        Burn { from, amount }.publish(&env);
    }

    pub fn clawback(env: Env, from: Address, amount: i128) {
        let admin = read_admin(&env);
        admin.require_auth();
        require_non_negative(amount);
        spend_balance(&env, &from, amount);
        Clawback { from, amount }.publish(&env);
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        let admin = read_admin(&env);
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        SetAdmin { admin, new_admin }.publish(&env);
    }

    pub fn set_authorized(env: Env, id: Address, authorize: bool) {
        let admin = read_admin(&env);
        admin.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::State(id.clone()), &authorize);
        SetAuthorized {
            admin,
            id,
            authorize,
        }
        .publish(&env);
    }

    /// Returns whether `id` may send or receive tokens. Accounts are authorized by default.
    pub fn authorized(env: Env, id: Address) -> bool {
        read_authorized(&env, &id)
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        read_balance(&env, &id)
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        read_allowance(&env, &from, &spender).amount
    }

    pub fn decimals(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Decimals)
            .unwrap_or(7)
    }

    pub fn name(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Name)
            .unwrap_or_else(|| String::from_str(&env, "Test Token"))
    }

    pub fn symbol(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Symbol)
            .unwrap_or_else(|| String::from_str(&env, "TST"))
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Events},
        Env, IntoVal, Symbol,
    };

    #[test]
    fn test_token_flow() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        let contract_id = env.register(Token, ());
        let client = TokenClient::new(&env, &contract_id);

        client.initialize(
            &admin,
            &7,
            &String::from_str(&env, "Token"),
            &String::from_str(&env, "TKN"),
        );

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

        // Burn from
        client.approve(&user2, &user1, &50, &100);
        // Clear events from approve before checking burn_from
        env.events().all();
        client.burn_from(&user1, &user2, &50);

        assert_eq!(
            env.events().all(),
            soroban_sdk::vec![
                &env,
                (
                    contract_id.clone(),
                    (Symbol::new(&env, "burn"), user2.clone()).into_val(&env),
                    50_i128.into_val(&env)
                )
            ]
        );
        assert_eq!(client.balance(&user2), 350);
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
        client.initialize(
            &admin,
            &7,
            &String::from_str(&env, "Token"),
            &String::from_str(&env, "TKN"),
        );
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
        client.initialize(
            &admin,
            &7,
            &String::from_str(&env, "Token"),
            &String::from_str(&env, "TKN"),
        );

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
        client.initialize(
            &admin,
            &7,
            &String::from_str(&env, "Token"),
            &String::from_str(&env, "TKN"),
        );
        client.initialize(
            &admin,
            &7,
            &String::from_str(&env, "Token"),
            &String::from_str(&env, "TKN"),
        );
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
        client.initialize(
            &admin,
            &7,
            &String::from_str(&env, "Token"),
            &String::from_str(&env, "TKN"),
        );
        env.mock_all_auths();
        client.mint(&user1, &1000);
        client.approve(&user1, &user2, &100, &100);

        client.transfer_from(&user2, &user1, &user2, &101);
    }

    fn new_token(env: &Env) -> (TokenClient<'_>, Address) {
        let admin = Address::generate(env);
        let contract_id = env.register(Token, ());
        let client = TokenClient::new(env, &contract_id);
        client.initialize(
            &admin,
            &7,
            &String::from_str(env, "Token"),
            &String::from_str(env, "TKN"),
        );
        env.mock_all_auths();
        (client, admin)
    }

    #[test]
    #[should_panic(expected = "balance overflow")]
    fn test_mint_overflow_is_rejected() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let user = Address::generate(&env);
        client.mint(&user, &i128::MAX);
        client.mint(&user, &1);
    }

    #[test]
    fn test_expired_allowance_reads_as_zero() {
        use soroban_sdk::testutils::Ledger;
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let owner = Address::generate(&env);
        let spender = Address::generate(&env);

        client.approve(&owner, &spender, &100, &10);
        assert_eq!(client.allowance(&owner, &spender), 100);

        env.ledger().set_sequence_number(10);
        assert_eq!(client.allowance(&owner, &spender), 100);

        env.ledger().set_sequence_number(11);
        assert_eq!(client.allowance(&owner, &spender), 0);
    }

    #[test]
    #[should_panic(expected = "insufficient allowance")]
    fn test_transfer_from_with_expired_allowance() {
        use soroban_sdk::testutils::Ledger;
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let owner = Address::generate(&env);
        let spender = Address::generate(&env);
        client.mint(&owner, &1000);
        client.approve(&owner, &spender, &100, &10);

        env.ledger().set_sequence_number(11);
        client.transfer_from(&spender, &owner, &spender, &1);
    }

    #[test]
    #[should_panic(expected = "expiration ledger is in the past")]
    fn test_approve_with_past_expiration() {
        use soroban_sdk::testutils::Ledger;
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let owner = Address::generate(&env);
        let spender = Address::generate(&env);

        env.ledger().set_sequence_number(50);
        client.approve(&owner, &spender, &100, &49);
    }

    #[test]
    fn test_approve_zero_clears_allowance_even_with_past_expiration() {
        use soroban_sdk::testutils::Ledger;
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let owner = Address::generate(&env);
        let spender = Address::generate(&env);

        client.approve(&owner, &spender, &100, &100);
        env.ledger().set_sequence_number(50);
        client.approve(&owner, &spender, &0, &0);
        assert_eq!(client.allowance(&owner, &spender), 0);
    }

    #[test]
    fn test_accounts_are_authorized_by_default() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let user = Address::generate(&env);
        assert!(client.authorized(&user));
    }

    #[test]
    fn test_set_authorized_toggles_flag_and_emits_event() {
        let env = Env::default();
        let (client, admin) = new_token(&env);
        let user = Address::generate(&env);

        client.set_authorized(&user, &false);
        assert_eq!(
            env.events().all(),
            soroban_sdk::vec![
                &env,
                (
                    client.address.clone(),
                    (
                        Symbol::new(&env, "set_authorized"),
                        admin.clone(),
                        user.clone()
                    )
                        .into_val(&env),
                    false.into_val(&env)
                )
            ]
        );

        assert!(!client.authorized(&user));

        client.set_authorized(&user, &true);
        assert!(client.authorized(&user));
    }

    #[test]
    #[should_panic(expected = "account is not authorized")]
    fn test_deauthorized_sender_cannot_transfer() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let (a, b) = (Address::generate(&env), Address::generate(&env));
        client.mint(&a, &100);
        client.set_authorized(&a, &false);
        client.transfer(&a, &b, &10);
    }

    #[test]
    #[should_panic(expected = "account is not authorized")]
    fn test_deauthorized_recipient_cannot_receive() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let (a, b) = (Address::generate(&env), Address::generate(&env));
        client.mint(&a, &100);
        client.set_authorized(&b, &false);
        client.transfer(&a, &b, &10);
    }

    #[test]
    #[should_panic(expected = "account is not authorized")]
    fn test_cannot_mint_to_deauthorized_account() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let user = Address::generate(&env);
        client.set_authorized(&user, &false);
        client.mint(&user, &1);
    }

    #[test]
    #[should_panic(expected = "account is not authorized")]
    fn test_transfer_from_respects_authorization() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let (owner, spender) = (Address::generate(&env), Address::generate(&env));
        client.mint(&owner, &100);
        client.approve(&owner, &spender, &50, &100);
        client.set_authorized(&owner, &false);
        client.transfer_from(&spender, &owner, &spender, &10);
    }

    #[test]
    fn test_reauthorized_account_can_transfer_again() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let (a, b) = (Address::generate(&env), Address::generate(&env));
        client.mint(&a, &100);
        client.set_authorized(&a, &false);
        client.set_authorized(&a, &true);
        client.transfer(&a, &b, &10);
        assert_eq!(client.balance(&b), 10);
    }

    #[test]
    fn test_burn_from_spends_allowance_and_balance() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let (owner, spender) = (Address::generate(&env), Address::generate(&env));
        client.mint(&owner, &500);
        client.approve(&owner, &spender, &200, &100);

        client.burn_from(&spender, &owner, &150);

        assert_eq!(client.balance(&owner), 350);
        assert_eq!(client.allowance(&owner, &spender), 50);
    }

    #[test]
    #[should_panic(expected = "insufficient allowance")]
    fn test_burn_from_above_allowance() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let (owner, spender) = (Address::generate(&env), Address::generate(&env));
        client.mint(&owner, &500);
        client.approve(&owner, &spender, &10, &100);
        client.burn_from(&spender, &owner, &11);
    }

    #[test]
    fn test_clawback_reduces_balance_and_emits_event() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let user = Address::generate(&env);
        client.mint(&user, &100);

        client.clawback(&user, &40);
        assert_eq!(
            env.events().all(),
            soroban_sdk::vec![
                &env,
                (
                    client.address.clone(),
                    (Symbol::new(&env, "clawback"), user.clone()).into_val(&env),
                    40_i128.into_val(&env)
                )
            ]
        );
        assert_eq!(client.balance(&user), 60);
    }

    #[test]
    #[should_panic(expected = "insufficient balance")]
    fn test_clawback_above_balance() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let user = Address::generate(&env);
        client.mint(&user, &10);
        client.clawback(&user, &11);
    }

    #[test]
    #[should_panic]
    fn test_clawback_requires_admin_auth() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        let id = env.register(Token, ());
        let client = TokenClient::new(&env, &id);
        client.initialize(
            &admin,
            &7,
            &String::from_str(&env, "Token"),
            &String::from_str(&env, "TKN"),
        );
        client.clawback(&user, &1);
    }

    #[test]
    fn test_set_admin_hands_over_minting_rights() {
        let env = Env::default();
        let (client, admin) = new_token(&env);
        let new_admin = Address::generate(&env);

        client.set_admin(&new_admin);
        assert_eq!(
            env.events().all(),
            soroban_sdk::vec![
                &env,
                (
                    client.address.clone(),
                    (Symbol::new(&env, "set_admin"), admin.clone()).into_val(&env),
                    new_admin.clone().into_val(&env)
                )
            ]
        );

        client.mint(&new_admin, &5);
        assert_eq!(env.auths().last().unwrap().0, new_admin);
        assert_eq!(client.balance(&new_admin), 5);
    }

    #[test]
    fn test_metadata_matches_initialization() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let id = env.register(Token, ());
        let client = TokenClient::new(&env, &id);
        client.initialize(
            &admin,
            &18,
            &String::from_str(&env, "Mock USDC"),
            &String::from_str(&env, "USDC"),
        );

        assert_eq!(client.decimals(), 18);
        assert_eq!(client.name(), String::from_str(&env, "Mock USDC"));
        assert_eq!(client.symbol(), String::from_str(&env, "USDC"));
    }

    #[test]
    fn test_metadata_defaults_when_uninitialized() {
        let env = Env::default();
        let id = env.register(Token, ());
        let client = TokenClient::new(&env, &id);

        assert_eq!(client.decimals(), 7);
        assert_eq!(client.name(), String::from_str(&env, "Test Token"));
        assert_eq!(client.symbol(), String::from_str(&env, "TST"));
    }

    #[test]
    fn test_negative_amounts_are_rejected_everywhere() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let (a, b) = (Address::generate(&env), Address::generate(&env));
        client.mint(&a, &100);

        assert!(client.try_transfer(&a, &b, &-1).is_err());
        assert!(client.try_approve(&a, &b, &-1, &100).is_err());
        assert!(client.try_mint(&a, &-1).is_err());
        assert!(client.try_burn(&a, &-1).is_err());
        assert!(client.try_clawback(&a, &-1).is_err());
        assert!(client.try_transfer_from(&b, &a, &b, &-1).is_err());
        assert!(client.try_burn_from(&b, &a, &-1).is_err());
        assert_eq!(client.balance(&a), 100);
    }

    #[test]
    fn test_zero_amount_transfer_is_a_noop() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let (a, b) = (Address::generate(&env), Address::generate(&env));
        client.mint(&a, &10);
        client.transfer(&a, &b, &0);
        assert_eq!(client.balance(&a), 10);
        assert_eq!(client.balance(&b), 0);
    }

    #[test]
    fn test_self_transfer_keeps_balance() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let a = Address::generate(&env);
        client.mint(&a, &10);
        client.transfer(&a, &a, &10);
        assert_eq!(client.balance(&a), 10);
    }

    #[test]
    fn test_approve_overwrites_previous_allowance() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let (owner, spender) = (Address::generate(&env), Address::generate(&env));
        client.approve(&owner, &spender, &100, &50);
        client.approve(&owner, &spender, &30, &60);
        assert_eq!(client.allowance(&owner, &spender), 30);
    }

    #[test]
    fn test_allowances_are_scoped_per_spender() {
        let env = Env::default();
        let (client, _admin) = new_token(&env);
        let owner = Address::generate(&env);
        let (s1, s2) = (Address::generate(&env), Address::generate(&env));
        client.approve(&owner, &s1, &100, &50);
        assert_eq!(client.allowance(&owner, &s1), 100);
        assert_eq!(client.allowance(&owner, &s2), 0);
    }

    #[test]
    #[should_panic(expected = "not initialized")]
    fn test_mint_before_initialize_panics_clearly() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register(Token, ());
        let client = TokenClient::new(&env, &id);
        client.mint(&Address::generate(&env), &1);
    }
}
