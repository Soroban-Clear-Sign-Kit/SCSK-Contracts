#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{
    IntoVal, symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    token::{StellarAssetClient, TokenClient},
    vec,
};

struct Setup<'a> {
    env: Env,
    client: ForwarderClient<'a>,
    contract_id: Address,
    token: TokenClient<'a>,
    from: Address,
    to: Address,
}

fn setup(initial_balance: i128) -> Setup<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin);
    let token = TokenClient::new(&env, &sac.address());

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    StellarAssetClient::new(&env, &sac.address()).mint(&from, &initial_balance);

    let contract_id = env.register(Forwarder, ());
    let client = ForwarderClient::new(&env, &contract_id);

    Setup {
        env,
        client,
        contract_id,
        token,
        from,
        to,
    }
}

#[test]
fn forward_moves_tokens_between_accounts() {
    let s = setup(1_000);

    s.client.forward(&s.from, &s.token.address, &s.to, &400);

    assert_eq!(s.token.balance(&s.from), 600);
    assert_eq!(s.token.balance(&s.to), 400);
}

#[test]
fn forward_requires_sender_auth_with_nested_token_transfer() {
    let s = setup(1_000);

    s.client.forward(&s.from, &s.token.address, &s.to, &250);

    assert_eq!(
        s.env.auths(),
        std::vec![(
            s.from.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    s.contract_id.clone(),
                    symbol_short!("forward"),
                    (
                        s.from.clone(),
                        s.token.address.clone(),
                        s.to.clone(),
                        250_i128
                    )
                        .into_val(&s.env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        s.token.address.clone(),
                        symbol_short!("transfer"),
                        (s.from.clone(), s.to.clone(), 250_i128).into_val(&s.env),
                    )),
                    sub_invocations: std::vec![],
                }],
            }
        )]
    );
}

#[test]
fn forward_with_meta_moves_tokens_for_every_mode() {
    let s = setup(1_000);
    let meta = Meta {
        memo: String::from_str(&s.env, "invoice #42"),
        tags: vec![&s.env, symbol_short!("rent"), symbol_short!("monthly")],
    };

    s.client
        .forward_with_meta(&s.from, &s.token.address, &s.to, &100, &meta, &Mode::Fast);
    s.client.forward_with_meta(
        &s.from,
        &s.token.address,
        &s.to,
        &150,
        &meta,
        &Mode::Scheduled(1_700_000_000),
    );

    assert_eq!(s.token.balance(&s.from), 750);
    assert_eq!(s.token.balance(&s.to), 250);
}

#[test]
#[should_panic]
fn forward_fails_without_sender_auth() {
    let s = setup(1_000);
    s.env.set_auths(&[]);

    s.client.forward(&s.from, &s.token.address, &s.to, &1);
}

#[test]
fn forward_rejects_amount_above_balance() {
    let s = setup(100);

    let result = s.client.try_forward(&s.from, &s.token.address, &s.to, &101);

    assert!(result.is_err());
    assert_eq!(s.token.balance(&s.from), 100);
    assert_eq!(s.token.balance(&s.to), 0);
}

#[test]
fn forward_rejects_negative_amount() {
    let s = setup(100);

    let result = s.client.try_forward(&s.from, &s.token.address, &s.to, &-1);

    assert!(result.is_err());
    assert_eq!(s.token.balance(&s.from), 100);
}

#[test]
fn forward_for_args_requires_custom_args_with_nested_token_transfer() {
    let s = setup(1_000);

    s.client
        .forward_for_args(&s.from, &s.token.address, &s.to, &250);

    assert_eq!(
        s.env.auths(),
        std::vec![(
            s.from.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    s.contract_id.clone(),
                    Symbol::new(&s.env, "forward_for_args"),
                    (
                        symbol_short!("forw_args"),
                        s.token.address.clone(),
                        s.to.clone(),
                        250_i128,
                    )
                        .into_val(&s.env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        s.token.address.clone(),
                        symbol_short!("transfer"),
                        (s.from.clone(), s.to.clone(), 250_i128).into_val(&s.env),
                    )),
                    sub_invocations: std::vec![],
                }],
            }
        )]
    );

    assert_eq!(s.token.balance(&s.from), 750);
    assert_eq!(s.token.balance(&s.to), 250);
}

#[test]
fn chain_call_creates_three_level_sub_invocation_tree() {
    let s = setup(1_000);
    let relay_id = s.env.register(Forwarder, ());

    s.client
        .chain_call(&s.from, &relay_id, &s.token.address, &s.to, &300);

    assert_eq!(
        s.env.auths(),
        std::vec![(
            s.from.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    s.contract_id.clone(),
                    Symbol::new(&s.env, "chain_call"),
                    (
                        s.from.clone(),
                        relay_id.clone(),
                        s.token.address.clone(),
                        s.to.clone(),
                        300_i128,
                    )
                        .into_val(&s.env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        relay_id.clone(),
                        Symbol::new(&s.env, "chain_step"),
                        (
                            s.from.clone(),
                            s.token.address.clone(),
                            s.to.clone(),
                            300_i128,
                        )
                            .into_val(&s.env),
                    )),
                    sub_invocations: std::vec![AuthorizedInvocation {
                        function: AuthorizedFunction::Contract((
                            s.token.address.clone(),
                            symbol_short!("transfer"),
                            (s.from.clone(), s.to.clone(), 300_i128).into_val(&s.env),
                        )),
                        sub_invocations: std::vec![],
                    }],
                }],
            }
        )]
    );

    assert_eq!(s.token.balance(&s.from), 700);
    assert_eq!(s.token.balance(&s.to), 300);
}

#[test]
fn dual_depth_auth_records_same_address_at_two_depths() {
    let s = setup(1_000);
    let sub_id = s.env.register(Forwarder, ());

    s.client.dual_depth_auth(&s.from, &sub_id, &500);

    assert_eq!(
        s.env.auths(),
        std::vec![(
            s.from.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    s.contract_id.clone(),
                    Symbol::new(&s.env, "dual_depth_auth"),
                    (s.from.clone(), sub_id.clone(), 500_i128).into_val(&s.env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        sub_id.clone(),
                        Symbol::new(&s.env, "inner_action"),
                        (s.from.clone(), 500_i128).into_val(&s.env),
                    )),
                    sub_invocations: std::vec![],
                }],
            }
        )]
    );
}

#[test]
fn authorize_as_current_contract_delegates_to_relay() {
    let s = setup(1_000);
    let relay_id = s.env.register(Forwarder, ());

    // Mint tokens directly to the calling contract (Forwarder)
    StellarAssetClient::new(&s.env, &s.token.address).mint(&s.contract_id, &500);

    s.client
        .contract_transfer(&s.from, &relay_id, &s.token.address, &s.to, &150);

    assert_eq!(
        s.env.auths(),
        std::vec![(
            s.from.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    s.contract_id.clone(),
                    Symbol::new(&s.env, "contract_transfer"),
                    (
                        s.from.clone(),
                        relay_id.clone(),
                        s.token.address.clone(),
                        s.to.clone(),
                        150_i128,
                    )
                        .into_val(&s.env),
                )),
                sub_invocations: std::vec![],
            }
        )]
    );

    assert_eq!(s.token.balance(&s.contract_id), 350);
    assert_eq!(s.token.balance(&s.to), 150);
}

#[test]
fn custom_account_authorizes_via_check_auth() {
    let s = setup(1_000);

    let custom_acc = s.env.register(CustomAccount, ());
    StellarAssetClient::new(&s.env, &s.token.address).mint(&custom_acc, &600);

    s.client.forward(&custom_acc, &s.token.address, &s.to, &200);

    assert_eq!(
        s.env.auths(),
        std::vec![(
            custom_acc.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    s.contract_id.clone(),
                    symbol_short!("forward"),
                    (
                        custom_acc.clone(),
                        s.token.address.clone(),
                        s.to.clone(),
                        200_i128,
                    )
                        .into_val(&s.env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        s.token.address.clone(),
                        symbol_short!("transfer"),
                        (custom_acc.clone(), s.to.clone(), 200_i128).into_val(&s.env),
                    )),
                    sub_invocations: std::vec![],
                }],
            }
        )]
    );

    assert_eq!(s.token.balance(&custom_acc), 400);
    assert_eq!(s.token.balance(&s.to), 200);
}
