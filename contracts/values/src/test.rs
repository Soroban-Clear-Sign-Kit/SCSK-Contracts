#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{
    Env, IntoVal,
    testutils::{Address as _, Events},
    vec,
};

fn setup(env: &Env) -> (ValuesContractClient<'_>, Address) {
    let contract_id = env.register(ValuesContract, ());
    let client = ValuesContractClient::new(env, &contract_id);
    (client, contract_id)
}

#[test]
fn test_echo_nested_vec() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let inner1 = vec![
        &env,
        String::from_str(&env, "alpha"),
        String::from_str(&env, "beta"),
    ];
    let inner2 = vec![&env, String::from_str(&env, "gamma")];
    let val: Vec<Vec<String>> = vec![&env, inner1, inner2];

    let result = client.echo_nested_vec(&tag, &val);
    assert_eq!(result, val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "nested_vec"),
                    tag,
                )
                    .into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_nested_map() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let mut inner = Map::new(&env);
    inner.set(String::from_str(&env, "item1"), 100_i128);
    inner.set(String::from_str(&env, "item2"), -50_i128);

    let mut val = Map::new(&env);
    val.set(Symbol::new(&env, "vault"), inner);

    let result = client.echo_nested_map(&tag, &val);
    assert_eq!(result, val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "nested_map"),
                    tag,
                )
                    .into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_tuple() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");
    let addr = Address::generate(&env);

    let val = (addr, Symbol::new(&env, "TUPLE"), 123456789_i128, true);

    let result = client.echo_tuple(&tag, &val);
    assert_eq!(result, val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "tuple"),
                    tag,
                    true,
                )
                    .into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_option_some() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let some_val = Some(String::from_str(&env, "hello_option"));
    let res_some = client.echo_option(&tag, &some_val);
    assert_eq!(res_some, some_val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "option"),
                    tag,
                )
                    .into_val(&env),
                some_val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_option_none() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let none_val: Option<String> = None;
    let res_none = client.echo_option(&tag, &none_val);
    assert_eq!(res_none, none_val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "option"),
                    tag,
                )
                    .into_val(&env),
                none_val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_enum() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");
    let addr = Address::generate(&env);

    // 1. Empty variant
    let v1 = CustomEnum::Empty;
    assert_eq!(client.echo_enum(&tag, &v1), v1);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "enum"),
                    tag.clone()
                )
                    .into_val(&env),
                v1.into_val(&env),
            )
        ]
    );

    // 2. Value variant
    let v2 = CustomEnum::Value(999);
    assert_eq!(client.echo_enum(&tag, &v2), v2);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "enum"),
                    tag.clone()
                )
                    .into_val(&env),
                v2.into_val(&env),
            )
        ]
    );

    // 3. Pair variant
    let v3 = CustomEnum::Pair(String::from_str(&env, "pair_str"), -777_i128);
    assert_eq!(client.echo_enum(&tag, &v3), v3);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "enum"),
                    tag.clone()
                )
                    .into_val(&env),
                v3.into_val(&env),
            )
        ]
    );

    // 4. Record variant
    let v4 = CustomEnum::Record(Symbol::new(&env, "rec_key"), addr);
    assert_eq!(client.echo_enum(&tag, &v4), v4);
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id,
                (Symbol::new(&env, "values"), Symbol::new(&env, "enum"), tag).into_val(&env),
                v4.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_bytes() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let val = Bytes::from_slice(&env, b"soroban binary payload");
    let result = client.echo_bytes(&tag, &val);
    assert_eq!(result, val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (Symbol::new(&env, "values"), Symbol::new(&env, "bytes"), tag,).into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_bytes_n() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let raw = [77u8; 32];
    let val = BytesN::from_array(&env, &raw);
    let result = client.echo_bytes_n(&tag, &val);
    assert_eq!(result, val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "bytes_n"),
                    tag,
                )
                    .into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_string() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let val = String::from_str(&env, "Soroban Clear Sign Kit String");
    let result = client.echo_string(&tag, &val);
    assert_eq!(result, val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "string"),
                    tag,
                )
                    .into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_symbol() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let val = Symbol::new(&env, "SYMBOL_VAL");
    let result = client.echo_symbol(&tag, &val);
    assert_eq!(result, val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "symbol"),
                    tag,
                )
                    .into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_address() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let val = Address::generate(&env);
    let result = client.echo_address(&tag, &val);
    assert_eq!(result, val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "address"),
                    tag,
                )
                    .into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_i128() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let val = -9876543210123456789_i128;
    let result = client.echo_i128(&tag, &val);
    assert_eq!(result, val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (Symbol::new(&env, "values"), Symbol::new(&env, "i128"), tag).into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_u128() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let val = 12345678901234567890_u128;
    let result = client.echo_u128(&tag, &val);
    assert_eq!(result, val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (Symbol::new(&env, "values"), Symbol::new(&env, "u128"), tag).into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_i256() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let val = I256::from_i128(&env, -8888888888888888888_i128);
    let result = client.echo_i256(&tag, &val);
    assert_eq!(result, val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (Symbol::new(&env, "values"), Symbol::new(&env, "i256"), tag).into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_u256() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let val = U256::from_u128(&env, 7777777777777777777_u128);
    let result = client.echo_u256(&tag, &val);
    assert_eq!(result, val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (Symbol::new(&env, "values"), Symbol::new(&env, "u256"), tag).into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_duration() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let val = Duration::from_seconds(&env, 86400);
    let result = client.echo_duration(&tag, &val);
    assert_eq!(result, val);
    assert_eq!(result.to_seconds(), 86400);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "duration"),
                    tag,
                )
                    .into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_timepoint() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let val = Timepoint::from_unix(&env, 1710000000);
    let result = client.echo_timepoint(&tag, &val);
    assert_eq!(result, val);
    assert_eq!(result.to_unix(), 1710000000);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "timepoint"),
                    tag,
                )
                    .into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}

#[test]
fn test_echo_struct() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let tag = Symbol::new(&env, "test_tag");

    let val = CustomStruct {
        id: 42,
        name: String::from_str(&env, "CustomStructName"),
        payload: Bytes::from_slice(&env, b"payload_bytes"),
    };

    let result = client.echo_struct(&tag, &val);
    assert_eq!(result, val);

    let events = env.events().all();
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                (
                    Symbol::new(&env, "values"),
                    Symbol::new(&env, "struct"),
                    tag,
                    42u32,
                )
                    .into_val(&env),
                val.into_val(&env),
            )
        ]
    );
}
