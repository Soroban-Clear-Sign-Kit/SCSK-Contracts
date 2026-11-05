#![cfg(test)]

use super::*;
use soroban_sdk::{
    Env, IntoVal, Symbol,
    testutils::{Address as _, Events},
    token::{StellarAssetClient, TokenClient},
    vec,
};

const DUMMY_UPGRADE_WASM: &[u8] = include_bytes!("../test_fixtures/dummy_upgrade.wasm");

struct Fixture<'a> {
    env: Env,
    client: RiskyContractClient<'a>,
    contract_id: Address,
    admin: Address,
    user: Address,
    spender: Address,
    token_address: Address,
    token_client: TokenClient<'a>,
    sac_client: StellarAssetClient<'a>,
}

fn setup_fixture() -> Fixture<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let spender = Address::generate(&env);

    let sac = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = sac.address();
    let token_client = TokenClient::new(&env, &token_address);
    let sac_client = StellarAssetClient::new(&env, &token_address);

    let contract_id = env.register(RiskyContract, ());
    let client = RiskyContractClient::new(&env, &contract_id);
    client.init(&admin);

    Fixture {
        env,
        client,
        contract_id,
        admin,
        user,
        spender,
        token_address,
        token_client,
        sac_client,
    }
}

#[test]
fn test_set_admin_success() {
    let f = setup_fixture();
    let new_admin = Address::generate(&f.env);

    assert_eq!(f.client.get_admin(), f.admin);
    f.client.set_admin(&new_admin);

    assert_eq!(
        f.env.events().all().filter_by_contract(&f.contract_id),
        vec![
            &f.env,
            (
                f.contract_id.clone(),
                (Symbol::new(&f.env, "admin_changed"), f.admin.clone()).into_val(&f.env),
                new_admin.clone().into_val(&f.env),
            ),
        ]
    );

    assert_eq!(f.client.get_admin(), new_admin);
}

#[test]
#[should_panic(expected = "already initialized")]
fn test_init_twice_panics() {
    let f = setup_fixture();
    let other_admin = Address::generate(&f.env);
    f.client.init(&other_admin);
}

#[test]
fn test_upgrade_contract() {
    let f = setup_fixture();
    let wasm = Bytes::from_slice(&f.env, DUMMY_UPGRADE_WASM);
    let wasm_hash = f.env.deployer().upload_contract_wasm(wasm);

    f.client.upgrade(&wasm_hash);

    assert_eq!(
        f.env.events().all().filter_by_contract(&f.contract_id),
        vec![
            &f.env,
            (
                f.contract_id.clone(),
                (Symbol::new(&f.env, "contract_upgraded"), f.admin.clone()).into_val(&f.env),
                wasm_hash.into_val(&f.env),
            ),
        ]
    );
}

#[test]
fn test_unlimited_approve() {
    let f = setup_fixture();
    f.sac_client.mint(&f.user, &10_000);

    assert_eq!(f.token_client.allowance(&f.user, &f.spender), 0);

    f.client
        .unlimited_approve(&f.token_address, &f.user, &f.spender, &1_000);

    assert_eq!(
        f.env.events().all().filter_by_contract(&f.contract_id),
        vec![
            &f.env,
            (
                f.contract_id.clone(),
                (
                    Symbol::new(&f.env, "unlimited_approve"),
                    f.user.clone(),
                    f.spender.clone(),
                )
                    .into_val(&f.env),
                i128::MAX.into_val(&f.env),
            ),
        ]
    );

    assert_eq!(f.token_client.allowance(&f.user, &f.spender), i128::MAX);
}

#[test]
fn test_approve_fresh_spender() {
    let f = setup_fixture();
    f.sac_client.mint(&f.user, &50_000);

    let fresh_spender = Address::generate(&f.env);
    assert_eq!(f.token_client.allowance(&f.user, &fresh_spender), 0);

    f.client
        .approve_fresh_spender(&f.token_address, &f.user, &fresh_spender, &25_000, &500);

    assert_eq!(
        f.env.events().all().filter_by_contract(&f.contract_id),
        vec![
            &f.env,
            (
                f.contract_id.clone(),
                (
                    Symbol::new(&f.env, "fresh_spender_approved"),
                    f.user.clone(),
                    fresh_spender.clone(),
                )
                    .into_val(&f.env),
                25_000_i128.into_val(&f.env),
            ),
        ]
    );

    assert_eq!(f.token_client.allowance(&f.user, &fresh_spender), 25_000);
}

#[test]
fn test_large_transfer() {
    let f = setup_fixture();
    let huge_amount = 100_000_000_000_000_000_i128; // 100 quadrillion units
    let recipient = Address::generate(&f.env);

    f.sac_client.mint(&f.user, &huge_amount);
    assert_eq!(f.token_client.balance(&f.user), huge_amount);
    assert_eq!(f.token_client.balance(&recipient), 0);

    f.client
        .large_transfer(&f.token_address, &f.user, &recipient, &huge_amount);

    assert_eq!(
        f.env.events().all().filter_by_contract(&f.contract_id),
        vec![
            &f.env,
            (
                f.contract_id.clone(),
                (
                    Symbol::new(&f.env, "large_transfer_executed"),
                    f.user.clone(),
                    recipient.clone(),
                )
                    .into_val(&f.env),
                huge_amount.into_val(&f.env),
            ),
        ]
    );

    assert_eq!(f.token_client.balance(&f.user), 0);
    assert_eq!(f.token_client.balance(&recipient), huge_amount);
}

#[test]
fn test_oversize_payload() {
    let f = setup_fixture();
    // 32 KB payload
    let raw = [0xabu8; 32768];
    let payload = Bytes::from_slice(&f.env, &raw);

    let returned_len = f.client.oversize_payload(&payload);
    assert_eq!(returned_len, 32768);

    assert_eq!(
        f.env.events().all().filter_by_contract(&f.contract_id),
        vec![
            &f.env,
            (
                f.contract_id.clone(),
                (Symbol::new(&f.env, "oversize_payload_processed"),).into_val(&f.env),
                32768_u32.into_val(&f.env),
            ),
        ]
    );
}

#[test]
fn test_deeply_nested_structure() {
    let f = setup_fixture();

    let inner = vec![&f.env, 1u32, 2u32, 3u32];
    let l3 = vec![&f.env, inner.clone(), inner.clone()];
    let l2 = vec![&f.env, l3.clone(), l3.clone()];
    let levels = vec![&f.env, l2.clone(), l2.clone()];

    // 2 * 2 * 2 * 3 = 24 elements of values (1+2+3=6), total = 8 * 6 = 48
    let total = f.client.deeply_nested_structure(&levels);
    assert_eq!(total, 48);

    assert_eq!(
        f.env.events().all().filter_by_contract(&f.contract_id),
        vec![
            &f.env,
            (
                f.contract_id.clone(),
                (Symbol::new(&f.env, "deeply_nested_processed"),).into_val(&f.env),
                48_u32.into_val(&f.env),
            ),
        ]
    );
}
