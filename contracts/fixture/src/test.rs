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
