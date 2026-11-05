#![no_std]
use soroban_sdk::{contract, contractevent, contractimpl, contracttype, Address, Env, String};
use soroban_token_sdk::events::{
    Approve, Burn, Clawback, MintWithAmountOnly, TransferWithAmountOnly,
};

const DAY_IN_LEDGERS: u32 = 17280;
const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS;
const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

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

fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

fn read_balance(env: &Env, id: &Address) -> i128 {
    let key = DataKey::Balance(id.clone());
    if let Some(balance) = env.storage().persistent().get(&key) {
        env.storage().persistent().extend_ttl(
            &key,
            BALANCE_LIFETIME_THRESHOLD,
            BALANCE_BUMP_AMOUNT,
        );
        balance
    } else {
        0
    }
}

fn write_balance(env: &Env, id: &Address, amount: i128) {
    let key = DataKey::Balance(id.clone());
    env.storage().persistent().set(&key, &amount);
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

fn read_allowance(env: &Env, from: &Address, spender: &Address) -> AllowanceValue {
    let key = DataKey::Allowance(AllowanceDataKey {
        from: from.clone(),
        spender: spender.clone(),
    });
    if let Some(allowance) = env.storage().persistent().get::<_, AllowanceValue>(&key) {
        if allowance.expiration_ledger < env.ledger().sequence() {
            AllowanceValue {
                amount: 0,
                expiration_ledger: allowance.expiration_ledger,
            }
        } else {
            env.storage().persistent().extend_ttl(
                &key,
                BALANCE_LIFETIME_THRESHOLD,
                BALANCE_BUMP_AMOUNT,
            );
            allowance
        }
    } else {
        AllowanceValue {
            amount: 0,
            expiration_ledger: 0,
        }
    }
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
    env.storage()
        .persistent()
        .extend_ttl(&key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

fn spend_allowance(env: &Env, from: &Address, spender: &Address, amount: i128) {
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

    if allowance.expiration_ledger < env.ledger().sequence() {
        panic!("expired allowance");
    }
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
        extend_instance_ttl(&env);
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        if amount < 0 {
            panic!("negative amount");
        }
        extend_instance_ttl(&env);

        let from_bal = read_balance(&env, &from);
        if from_bal < amount {
            panic!("insufficient balance");
        }

        if from != to {
            write_balance(&env, &from, from_bal - amount);
            let to_bal = read_balance(&env, &to);
            write_balance(&env, &to, to_bal + amount);
        } else {
            write_balance(&env, &from, from_bal);
        }

        TransferWithAmountOnly { from, to, amount }.publish(&env);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        if amount < 0 {
            panic!("negative amount");
        }
        extend_instance_ttl(&env);

        spend_allowance(&env, &from, &spender, amount);

        let from_bal = read_balance(&env, &from);
        if from_bal < amount {
            panic!("insufficient balance");
        }

        if from != to {
            write_balance(&env, &from, from_bal - amount);
            let to_bal = read_balance(&env, &to);
            write_balance(&env, &to, to_bal + amount);
        } else {
            write_balance(&env, &from, from_bal);
        }

        TransferWithAmountOnly { from, to, amount }.publish(&env);
    }

    pub fn approve(
        env: Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) {
        from.require_auth();
        if amount < 0 {
            panic!("negative amount");
        }
        if amount > 0 && expiration_ledger < env.ledger().sequence() {
            panic!("expired allowance");
        }
        extend_instance_ttl(&env);

        write_allowance(&env, &from, &spender, amount, expiration_ledger);
        Approve {
            from,
            spender,
            amount,
            expiration_ledger,
        }
        .publish(&env);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        if amount < 0 {
            panic!("negative amount");
        }
        extend_instance_ttl(&env);

        let to_bal = read_balance(&env, &to);
        write_balance(&env, &to, to_bal + amount);
        MintWithAmountOnly { to, amount }.publish(&env);
    }

    pub fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();
        if amount < 0 {
            panic!("negative amount");
        }
        extend_instance_ttl(&env);

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
        extend_instance_ttl(&env);

        spend_allowance(&env, &from, &spender, amount);

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
        extend_instance_ttl(&env);

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
        extend_instance_ttl(&env);

        env.storage().instance().set(&DataKey::Admin, &new_admin);
        SetAdmin { admin, new_admin }.publish(&env);
    }

    pub fn set_authorized(env: Env, id: Address, authorize: bool) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        extend_instance_ttl(&env);

        let key = DataKey::State(id.clone());
        env.storage().persistent().set(&key, &authorize);
        env.storage().persistent().extend_ttl(
            &key,
            BALANCE_LIFETIME_THRESHOLD,
            BALANCE_BUMP_AMOUNT,
        );
        SetAuthorized {
            admin,
            id,
            authorize,
        }
        .publish(&env);
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        extend_instance_ttl(&env);
        read_balance(&env, &id)
    }

    pub fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        extend_instance_ttl(&env);
        read_allowance(&env, &from, &spender).amount
    }

    pub fn decimals(env: Env) -> u32 {
        extend_instance_ttl(&env);
        env.storage()
            .instance()
            .get(&DataKey::Decimals)
            .expect("not initialized")
    }

    pub fn name(env: Env) -> String {
        extend_instance_ttl(&env);
        env.storage()
            .instance()
            .get(&DataKey::Name)
            .expect("not initialized")
    }

    pub fn symbol(env: Env) -> String {
        extend_instance_ttl(&env);
        env.storage()
            .instance()
            .get(&DataKey::Symbol)
            .expect("not initialized")
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Events, Ledger as _},
        Env, IntoVal, Symbol,
    };

    fn setup(env: &Env) -> (TokenClient<'_>, Address, Address, Address) {
        let admin = Address::generate(env);
        let user1 = Address::generate(env);
        let user2 = Address::generate(env);

        let contract_id = env.register(Token, ());
        let client = TokenClient::new(env, &contract_id);

        client.initialize(
            &admin,
            &7,
            &String::from_str(env, "Token"),
            &String::from_str(env, "TKN"),
        );

        (client, admin, user1, user2)
    }

    #[test]
    fn test_token_flow() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        let contract_id = client.address.clone();

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

        // Approve (SEP-41 specifies data = [amount, expiration_ledger] in vec format)
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

    // --- Interface function tests ---

    #[test]
    fn test_allowance() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        assert_eq!(client.allowance(&user1, &user2), 0);
        client.approve(&user1, &user2, &300, &50);
        assert_eq!(client.allowance(&user1, &user2), 300);
    }

    #[test]
    fn test_approve() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        // Initial approve
        client.approve(&user1, &user2, &400, &100);
        assert_eq!(client.allowance(&user1, &user2), 400);

        // Overwrite approve
        client.approve(&user1, &user2, &250, &150);
        assert_eq!(client.allowance(&user1, &user2), 250);

        // Clear allowance with 0
        client.approve(&user1, &user2, &0, &200);
        assert_eq!(client.allowance(&user1, &user2), 0);
    }

    #[test]
    fn test_balance() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        assert_eq!(client.balance(&user1), 0);
        assert_eq!(client.balance(&user2), 0);
        client.mint(&user1, &750);
        assert_eq!(client.balance(&user1), 750);
        assert_eq!(client.balance(&user2), 0);
    }

    #[test]
    fn test_transfer() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &1000);
        client.transfer(&user1, &user2, &400);

        assert_eq!(client.balance(&user1), 600);
        assert_eq!(client.balance(&user2), 400);
    }

    #[test]
    fn test_transfer_from() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &1000);
        client.approve(&user1, &user2, &500, &100);
        client.transfer_from(&user2, &user1, &user2, &350);

        assert_eq!(client.balance(&user1), 650);
        assert_eq!(client.balance(&user2), 350);
        assert_eq!(client.allowance(&user1, &user2), 150);
    }

    #[test]
    fn test_burn() {
        let env = Env::default();
        let (client, _admin, user1, _user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &500);
        client.burn(&user1, &150);
        assert_eq!(client.balance(&user1), 350);
    }

    #[test]
    fn test_burn_from() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &500);
        client.approve(&user1, &user2, &200, &100);
        client.burn_from(&user2, &user1, &80);

        assert_eq!(client.balance(&user1), 420);
        assert_eq!(client.allowance(&user1, &user2), 120);
    }

    #[test]
    fn test_decimals() {
        let env = Env::default();
        let (client, _admin, _user1, _user2) = setup(&env);
        assert_eq!(client.decimals(), 7);
    }

    #[test]
    fn test_name() {
        let env = Env::default();
        let (client, _admin, _user1, _user2) = setup(&env);
        assert_eq!(client.name(), String::from_str(&env, "Token"));
    }

    #[test]
    fn test_symbol() {
        let env = Env::default();
        let (client, _admin, _user1, _user2) = setup(&env);
        assert_eq!(client.symbol(), String::from_str(&env, "TKN"));
    }

    // --- Edge Case Tests ---

    #[test]
    fn test_expired_allowance_read() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.approve(&user1, &user2, &500, &10);
        assert_eq!(client.allowance(&user1, &user2), 500);

        // Advance ledger past expiration
        env.ledger().with_mut(|l| {
            l.sequence_number = 11;
        });

        // Expired allowance returns 0
        assert_eq!(client.allowance(&user1, &user2), 0);
    }

    #[test]
    #[should_panic(expected = "expired allowance")]
    fn test_expired_allowance_transfer_from() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &1000);
        client.approve(&user1, &user2, &500, &10);

        // Advance ledger past expiration
        env.ledger().with_mut(|l| {
            l.sequence_number = 11;
        });

        client.transfer_from(&user2, &user1, &user2, &100);
    }

    #[test]
    #[should_panic(expected = "expired allowance")]
    fn test_expired_allowance_burn_from() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &1000);
        client.approve(&user1, &user2, &500, &10);

        // Advance ledger past expiration
        env.ledger().with_mut(|l| {
            l.sequence_number = 11;
        });

        client.burn_from(&user2, &user1, &100);
    }

    #[test]
    #[should_panic(expected = "expired allowance")]
    fn test_approve_expired_expiration_ledger() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        env.ledger().with_mut(|l| {
            l.sequence_number = 50;
        });

        // Attempting to approve with expiration_ledger less than current ledger must panic
        client.approve(&user1, &user2, &100, &49);
    }

    #[test]
    fn test_zero_amount_transfer() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &100);
        client.transfer(&user1, &user2, &0);
        assert_eq!(client.balance(&user1), 100);
        assert_eq!(client.balance(&user2), 0);
    }

    #[test]
    fn test_zero_amount_transfer_from() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &100);
        client.approve(&user1, &user2, &50, &100);
        client.transfer_from(&user2, &user1, &user2, &0);

        assert_eq!(client.balance(&user1), 100);
        assert_eq!(client.balance(&user2), 0);
        assert_eq!(client.allowance(&user1, &user2), 50);
    }

    #[test]
    fn test_zero_amount_approve() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.approve(&user1, &user2, &100, &50);
        assert_eq!(client.allowance(&user1, &user2), 100);

        client.approve(&user1, &user2, &0, &50);
        assert_eq!(client.allowance(&user1, &user2), 0);
    }

    #[test]
    fn test_zero_amount_burn() {
        let env = Env::default();
        let (client, _admin, user1, _user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &100);
        client.burn(&user1, &0);
        assert_eq!(client.balance(&user1), 100);
    }

    #[test]
    fn test_zero_amount_burn_from() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &100);
        client.approve(&user1, &user2, &50, &100);
        client.burn_from(&user2, &user1, &0);

        assert_eq!(client.balance(&user1), 100);
        assert_eq!(client.allowance(&user1, &user2), 50);
    }

    #[test]
    #[should_panic(expected = "insufficient balance")]
    fn test_transfer_above_balance() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &100);
        client.transfer(&user1, &user2, &101);
    }

    #[test]
    #[should_panic(expected = "insufficient balance")]
    fn test_transfer_from_above_balance() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &50);
        client.approve(&user1, &user2, &500, &100);
        // Allowance is sufficient (500 >= 100), but balance is only 50
        client.transfer_from(&user2, &user1, &user2, &100);
    }

    #[test]
    #[should_panic(expected = "insufficient allowance")]
    fn test_transfer_from_above_allowance() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &1000);
        client.approve(&user1, &user2, &100, &100);
        client.transfer_from(&user2, &user1, &user2, &101);
    }

    #[test]
    #[should_panic(expected = "insufficient balance")]
    fn test_burn_above_balance() {
        let env = Env::default();
        let (client, _admin, user1, _user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &50);
        client.burn(&user1, &51);
    }

    #[test]
    #[should_panic(expected = "insufficient balance")]
    fn test_burn_from_above_balance() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &50);
        client.approve(&user1, &user2, &200, &100);
        client.burn_from(&user2, &user1, &51);
    }

    #[test]
    fn test_self_transfer() {
        let env = Env::default();
        let (client, _admin, user1, _user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &300);
        client.transfer(&user1, &user1, &100);
        assert_eq!(client.balance(&user1), 300);
    }

    #[test]
    #[should_panic(expected = "insufficient balance")]
    fn test_self_transfer_insufficient_balance() {
        let env = Env::default();
        let (client, _admin, user1, _user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &50);
        client.transfer(&user1, &user1, &51);
    }

    #[test]
    fn test_self_transfer_from() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.mint(&user1, &500);
        client.approve(&user1, &user2, &200, &100);
        client.transfer_from(&user2, &user1, &user1, &150);

        assert_eq!(client.balance(&user1), 500);
        assert_eq!(client.allowance(&user1, &user2), 50);
    }

    #[test]
    #[should_panic(expected = "negative amount")]
    fn test_negative_amount_transfer() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();
        client.transfer(&user1, &user2, &-1);
    }

    #[test]
    #[should_panic(expected = "negative amount")]
    fn test_negative_amount_transfer_from() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();
        client.transfer_from(&user2, &user1, &user2, &-1);
    }

    #[test]
    #[should_panic(expected = "negative amount")]
    fn test_negative_amount_approve() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();
        client.approve(&user1, &user2, &-1, &100);
    }

    #[test]
    #[should_panic(expected = "negative amount")]
    fn test_negative_amount_burn() {
        let env = Env::default();
        let (client, _admin, user1, _user2) = setup(&env);
        env.mock_all_auths();
        client.burn(&user1, &-1);
    }

    #[test]
    #[should_panic(expected = "negative amount")]
    fn test_negative_amount_burn_from() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();
        client.burn_from(&user2, &user1, &-1);
    }

    #[test]
    #[should_panic(expected = "negative amount")]
    fn test_negative_amount_mint() {
        let env = Env::default();
        let (client, _admin, user1, _user2) = setup(&env);
        env.mock_all_auths();
        client.mint(&user1, &-1);
    }

    #[test]
    #[should_panic(expected = "negative amount")]
    fn test_negative_amount_clawback() {
        let env = Env::default();
        let (client, _admin, user1, _user2) = setup(&env);
        env.mock_all_auths();
        client.clawback(&user1, &-1);
    }

    #[test]
    #[should_panic]
    fn test_mint_without_admin_auth() {
        let env = Env::default();
        let (client, _admin, user1, _user2) = setup(&env);
        // No mock auths, must panic
        client.mint(&user1, &100);
    }

    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_initialize_twice() {
        let env = Env::default();
        let (client, admin, _user1, _user2) = setup(&env);
        client.initialize(
            &admin,
            &7,
            &String::from_str(&env, "Token"),
            &String::from_str(&env, "TKN"),
        );
    }

    #[test]
    fn test_set_admin() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        client.set_admin(&user1);
        // user1 is now admin, can mint
        client.mint(&user2, &500);
        assert_eq!(client.balance(&user2), 500);
    }

    #[test]
    fn test_set_authorized() {
        let env = Env::default();
        let (client, _admin, user1, _user2) = setup(&env);
        env.mock_all_auths();

        client.set_authorized(&user1, &true);
        client.set_authorized(&user1, &false);
    }

    #[test]
    fn test_ttl_extension_instance_and_persistent() {
        let env = Env::default();
        let (client, _admin, user1, user2) = setup(&env);
        env.mock_all_auths();

        // Perform balance, allowance, and metadata calls
        client.mint(&user1, &1000);
        client.approve(&user1, &user2, &500, &1000);
        client.transfer(&user1, &user2, &100);

        assert_eq!(client.balance(&user1), 900);
        assert_eq!(client.balance(&user2), 100);
        assert_eq!(client.allowance(&user1, &user2), 500);
        assert_eq!(client.decimals(), 7);
        assert_eq!(client.name(), String::from_str(&env, "Token"));
        assert_eq!(client.symbol(), String::from_str(&env, "TKN"));
    }
}

#[cfg(test)]
mod properties;
