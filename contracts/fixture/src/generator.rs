//! Versioned, reproducible test fixture generator for SCSK-Contracts.
//!
//! Generates deterministic transaction envelopes, authorization entries, expected events,
//! and human-readable summaries for each scenario, updating `fixtures/manifest.json`.

extern crate std;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::string::{String, ToString};
use std::vec::Vec;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use soroban_sdk::testutils::{
    Address as _, AuthorizedFunction, AuthorizedInvocation, EnvTestConfig, Events as _,
};
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::xdr::{
    ContractEvent, ContractEventBody, ContractId, Hash, HostFunction, InvokeContractArgs,
    InvokeHostFunctionOp, Limits, Memo, MuxedAccount, Operation, OperationBody, Preconditions,
    ScAddress, ScSymbol, ScVal, ScVec, SequenceNumber, SorobanAddressCredentials,
    SorobanAuthorizationEntry, SorobanAuthorizedFunction, SorobanAuthorizedInvocation,
    SorobanCredentials, Transaction, TransactionEnvelope, TransactionExt, TransactionV1Envelope,
    Uint256, VecM, WriteXdr,
};
use soroban_sdk::{
    Address, Env, IntoVal, String as SorobanString, TryFromVal, Val, symbol_short, vec,
};

use crate::{CustomAccount, Forwarder, ForwarderClient, Meta, Mode};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ScenarioFixture {
    pub scenario_id: String,
    pub name: String,
    pub description: String,
    pub tx_envelope_xdr: String,
    pub auth_entries: Vec<AuthEntryFixture>,
    pub expected_events: Vec<EventFixture>,
    pub expected_summary: ExpectedSummary,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AuthEntryFixture {
    pub address: String,
    pub credentials_type: String,
    pub root_invocation: InvocationFixture,
    pub entry_xdr: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct InvocationFixture {
    pub contract: String,
    pub function: String,
    pub args: Vec<serde_json::Value>,
    pub sub_invocations: Vec<InvocationFixture>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct EventFixture {
    pub contract: String,
    pub topics: Vec<serde_json::Value>,
    pub data: serde_json::Value,
    pub event_xdr: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ExpectedSummary {
    pub intent: String,
    pub call_stack: Vec<String>,
    pub authorizers: Vec<AuthorizerSummary>,
    pub state_effects: Vec<String>,
    pub risk_level: String,
    pub warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AuthorizerSummary {
    pub address: String,
    pub scope: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub schema_version: String,
    pub files: BTreeMap<String, ManifestFileEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ManifestFileEntry {
    pub sha256: String,
    pub scenario_id: String,
    pub description: String,
}

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .canonicalize()
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures"))
}

fn format_address(addr: &Address) -> String {
    let s = addr.to_string();
    let mut buf = [0u8; 128];
    let len = s.len() as usize;
    s.copy_into_slice(&mut buf[..len]);
    String::from_utf8(buf[..len].to_vec()).expect("valid utf8 address")
}

fn scval_to_json(val: &ScVal) -> serde_json::Value {
    match val {
        ScVal::Bool(b) => serde_json::json!(b),
        ScVal::Void => serde_json::json!(null),
        ScVal::U32(n) => serde_json::json!(n),
        ScVal::I32(n) => serde_json::json!(n),
        ScVal::U64(n) => serde_json::json!(n.to_string()),
        ScVal::I64(n) => serde_json::json!(n.to_string()),
        ScVal::Timepoint(t) => serde_json::json!(t.0.to_string()),
        ScVal::Duration(d) => serde_json::json!(d.0.to_string()),
        ScVal::U128(parts) => {
            let v = ((parts.hi as u128) << 64) | (parts.lo as u128);
            serde_json::json!(v.to_string())
        }
        ScVal::I128(parts) => {
            let v = ((parts.hi as i128) << 64) | (parts.lo as i128);
            serde_json::json!(v.to_string())
        }
        ScVal::Bytes(b) => serde_json::json!(hex::encode(b.as_slice())),
        ScVal::String(s) => serde_json::json!(s.to_utf8_string_lossy()),
        ScVal::Symbol(s) => serde_json::json!(s.to_utf8_string_lossy()),
        ScVal::Vec(Some(v)) => {
            let items: Vec<serde_json::Value> = v.iter().map(scval_to_json).collect();
            serde_json::json!(items)
        }
        ScVal::Map(Some(m)) => {
            let entries: Vec<serde_json::Value> = m
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "key": scval_to_json(&e.key),
                        "val": scval_to_json(&e.val),
                    })
                })
                .collect();
            serde_json::json!(entries)
        }
        ScVal::Address(addr) => match addr {
            ScAddress::Account(acc) => {
                let bytes = match acc {
                    soroban_sdk::xdr::AccountId(
                        soroban_sdk::xdr::PublicKey::PublicKeyTypeEd25519(u),
                    ) => u.0,
                };
                serde_json::json!(std::format!("account:{}", hex::encode(bytes)))
            }
            ScAddress::Contract(ContractId(Hash(bytes))) => {
                serde_json::json!(std::format!("contract:{}", hex::encode(bytes)))
            }
            ScAddress::MuxedAccount(m) => serde_json::json!(std::format!("{:?}", m)),
            ScAddress::ClaimableBalance(c) => serde_json::json!(std::format!("{:?}", c)),
            _ => serde_json::json!(std::format!("{:?}", addr)),
        },
        _ => serde_json::json!(std::format!("{:?}", val)),
    }
}

fn to_scval<T: IntoVal<Env, Val>>(env: &Env, val: T) -> ScVal {
    let v: Val = val.into_val(env);
    ScVal::try_from_val(env, &v).expect("valid ScVal")
}

fn build_sc_address(env: &Env, addr: &Address) -> ScAddress {
    let sc_val = to_scval(env, addr.clone());
    match sc_val {
        ScVal::Address(sc_addr) => sc_addr,
        _ => panic!("expected ScVal::Address"),
    }
}

fn invocation_to_xdr(
    env: &Env,
    inv: &AuthorizedInvocation,
) -> (SorobanAuthorizedInvocation, InvocationFixture) {
    let AuthorizedFunction::Contract((contract_addr, fn_sym, args_val)) = &inv.function else {
        panic!("only contract invocations supported");
    };
    let contract_str = format_address(contract_addr);
    let fn_str = fn_sym.to_string();

    let sc_contract = build_sc_address(env, contract_addr);
    let val: Val = args_val.clone().into();
    let sc_args_val = ScVal::try_from_val(env, &val).expect("valid args ScVal");

    let sc_vec: ScVec = match sc_args_val {
        ScVal::Vec(Some(v)) => v,
        _ => soroban_sdk::xdr::ScVec(VecM::default()),
    };

    let args_json: Vec<serde_json::Value> = sc_vec.iter().map(scval_to_json).collect();

    let mut sub_xdrs = Vec::new();
    let mut sub_fixtures = Vec::new();
    for sub in &inv.sub_invocations {
        let (sub_xdr, sub_fix) = invocation_to_xdr(env, sub);
        sub_xdrs.push(sub_xdr);
        sub_fixtures.push(sub_fix);
    }

    let auth_inv = SorobanAuthorizedInvocation {
        function: SorobanAuthorizedFunction::ContractFn(InvokeContractArgs {
            contract_address: sc_contract,
            function_name: ScSymbol(fn_str.as_bytes().try_into().expect("valid symbol")),
            args: sc_vec.0,
        }),
        sub_invocations: sub_xdrs.try_into().expect("within sub_invocations limit"),
    };

    let fix = InvocationFixture {
        contract: contract_str,
        function: fn_str,
        args: args_json,
        sub_invocations: sub_fixtures,
    };

    (auth_inv, fix)
}

#[allow(clippy::too_many_arguments)]
fn build_fixture(
    env: &Env,
    scenario_id: &str,
    name: &str,
    description: &str,
    contract_addr: &Address,
    fn_name: &str,
    args: Vec<ScVal>,
    auths: &[(Address, AuthorizedInvocation)],
    events: &[ContractEvent],
    summary: ExpectedSummary,
) -> ScenarioFixture {
    let mut auth_fixtures = Vec::new();
    let mut auth_entries_xdr: Vec<SorobanAuthorizationEntry> = Vec::new();

    for (addr, root_inv) in auths {
        let (root_xdr, root_fixture) = invocation_to_xdr(env, root_inv);
        let sc_addr = build_sc_address(env, addr);

        let entry = SorobanAuthorizationEntry {
            credentials: SorobanCredentials::Address(SorobanAddressCredentials {
                address: sc_addr,
                nonce: 0,
                signature_expiration_ledger: 1000,
                signature: ScVal::Void,
            }),
            root_invocation: root_xdr,
        };

        let entry_b64 = entry
            .to_xdr_base64(Limits::none())
            .expect("valid entry XDR");

        auth_entries_xdr.push(entry);
        auth_fixtures.push(AuthEntryFixture {
            address: format_address(addr),
            credentials_type: "address".to_string(),
            root_invocation: root_fixture,
            entry_xdr: entry_b64,
        });
    }

    // Build root host function
    let sc_contract = build_sc_address(env, contract_addr);
    let op = Operation {
        source_account: None,
        body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
            host_function: HostFunction::InvokeContract(InvokeContractArgs {
                contract_address: sc_contract,
                function_name: ScSymbol(fn_name.as_bytes().try_into().expect("valid symbol")),
                args: args.try_into().expect("within args limits"),
            }),
            auth: auth_entries_xdr.try_into().expect("within auth limits"),
        }),
    };

    let tx = Transaction {
        source_account: MuxedAccount::Ed25519(Uint256([1u8; 32])),
        fee: 100,
        seq_num: SequenceNumber(1),
        cond: Preconditions::None,
        memo: Memo::None,
        operations: std::vec![op].try_into().expect("within ops limits"),
        ext: TransactionExt::V0,
    };

    let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
        tx,
        signatures: std::vec![].try_into().expect("empty signatures"),
    });

    let tx_envelope_xdr = envelope
        .to_xdr_base64(Limits::none())
        .expect("valid envelope XDR");

    let mut event_fixtures = Vec::new();
    for e in events {
        let contract_str = match &e.contract_id {
            Some(ContractId(Hash(bytes))) => hex::encode(bytes),
            None => "".to_string(),
        };

        let (topics_json, data_json) = match &e.body {
            ContractEventBody::V0(v0) => {
                let t: Vec<serde_json::Value> = v0.topics.iter().map(scval_to_json).collect();
                let d = scval_to_json(&v0.data);
                (t, d)
            }
        };

        let event_b64 = e.to_xdr_base64(Limits::none()).expect("valid event XDR");

        event_fixtures.push(EventFixture {
            contract: contract_str,
            topics: topics_json,
            data: data_json,
            event_xdr: event_b64,
        });
    }

    ScenarioFixture {
        scenario_id: scenario_id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        tx_envelope_xdr,
        auth_entries: auth_fixtures,
        expected_events: event_fixtures,
        expected_summary: summary,
    }
}

fn create_env() -> Env {
    let mut env = Env::default();
    env.set_config(EnvTestConfig {
        capture_snapshot_at_drop: false,
    });
    env.mock_all_auths();
    env
}

struct TestContext {
    env: Env,
    contract_id: Address,
    client: ForwarderClient<'static>,
    token_addr: Address,
    _token_client: TokenClient<'static>,
    from: Address,
    to: Address,
}

fn setup_context() -> TestContext {
    let env = create_env();
    let admin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(admin);
    let token_addr = sac.address();
    let token_client = TokenClient::new(&env, &token_addr);

    let from = Address::generate(&env);
    let to = Address::generate(&env);
    StellarAssetClient::new(&env, &token_addr).mint(&from, &10_000);

    let contract_id = env.register(Forwarder, ());
    let client = ForwarderClient::new(&env, &contract_id);

    TestContext {
        env,
        contract_id,
        client,
        token_addr,
        _token_client: token_client,
        from,
        to,
    }
}

pub fn generate_all_fixtures() -> (Vec<(String, ScenarioFixture)>, Manifest) {
    let mut scenarios = Vec::new();

    // 1. Standard require_auth
    {
        let ctx = setup_context();
        let amount = 250_i128;
        ctx.client
            .forward(&ctx.from, &ctx.token_addr, &ctx.to, &amount);

        let auths = ctx.env.auths();
        let events = ctx.env.events().all();
        let args = std::vec![
            to_scval(&ctx.env, ctx.from.clone()),
            to_scval(&ctx.env, ctx.token_addr.clone()),
            to_scval(&ctx.env, ctx.to.clone()),
            to_scval(&ctx.env, amount),
        ];

        let summary = ExpectedSummary {
            intent: std::format!(
                "Forward {} tokens from {} to {} via standard require_auth",
                amount,
                format_address(&ctx.from),
                format_address(&ctx.to)
            ),
            call_stack: std::vec![
                "Forwarder.forward(from, token, to, amount)".to_string(),
                "Token.transfer(from, to, amount)".to_string(),
            ],
            authorizers: std::vec![AuthorizerSummary {
                address: format_address(&ctx.from),
                scope: "Top-level invocation and downstream token transfer".to_string(),
            }],
            state_effects: std::vec![std::format!(
                "Transfer {} tokens from sender balance to recipient balance",
                amount
            )],
            risk_level: "low".to_string(),
            warnings: std::vec![],
        };

        let fix = build_fixture(
            &ctx.env,
            "01_standard_require_auth",
            "Standard Authorization: require_auth",
            "Baseline Soroban authorization where caller authorizes top-level entry point and nested token transfer",
            &ctx.contract_id,
            "forward",
            args,
            &auths,
            events.events(),
            summary,
        );
        scenarios.push(("01_standard_require_auth.json".to_string(), fix));
    }

    // 2. Custom args require_auth_for_args
    {
        let ctx = setup_context();
        let amount = 250_i128;
        ctx.client
            .forward_for_args(&ctx.from, &ctx.token_addr, &ctx.to, &amount);

        let auths = ctx.env.auths();
        let events = ctx.env.events().all();
        let args = std::vec![
            to_scval(&ctx.env, ctx.from.clone()),
            to_scval(&ctx.env, ctx.token_addr.clone()),
            to_scval(&ctx.env, ctx.to.clone()),
            to_scval(&ctx.env, amount),
        ];

        let summary = ExpectedSummary {
            intent: std::format!(
                "Forward {} tokens using decoupled require_auth_for_args authorization",
                amount
            ),
            call_stack: std::vec![
                "Forwarder.forward_for_args(from, token, to, amount)".to_string(),
                "Token.transfer(from, to, amount)".to_string(),
            ],
            authorizers: std::vec![AuthorizerSummary {
                address: format_address(&ctx.from),
                scope: "Custom parameters payload (forw_args, token, to, amount)".to_string(),
            }],
            state_effects: std::vec![std::format!(
                "Transfer {} tokens from sender to recipient",
                amount
            )],
            risk_level: "low".to_string(),
            warnings: std::vec![],
        };

        let fix = build_fixture(
            &ctx.env,
            "02_custom_args_auth",
            "Custom Arguments Authorization: require_auth_for_args",
            "Decoupled authorization where checked authorization parameters differ from entry point parameters",
            &ctx.contract_id,
            "forward_for_args",
            args,
            &auths,
            events.events(),
            summary,
        );
        scenarios.push(("02_custom_args_auth.json".to_string(), fix));
    }

    // 3. Three-level chain
    {
        let ctx = setup_context();
        let relay_id = ctx.env.register(Forwarder, ());
        let amount = 300_i128;
        ctx.client
            .chain_call(&ctx.from, &relay_id, &ctx.token_addr, &ctx.to, &amount);

        let auths = ctx.env.auths();
        let events = ctx.env.events().all();
        let args = std::vec![
            to_scval(&ctx.env, ctx.from.clone()),
            to_scval(&ctx.env, relay_id.clone()),
            to_scval(&ctx.env, ctx.token_addr.clone()),
            to_scval(&ctx.env, ctx.to.clone()),
            to_scval(&ctx.env, amount),
        ];

        let summary = ExpectedSummary {
            intent:
                "Cascading authorization through a 3-contract stack (Forwarder -> Relay -> Token)"
                    .to_string(),
            call_stack: std::vec![
                "Forwarder.chain_call(from, relay, token, to, amount)".to_string(),
                "Relay.chain_step(from, token, to, amount)".to_string(),
                "Token.transfer(from, to, amount)".to_string(),
            ],
            authorizers: std::vec![AuthorizerSummary {
                address: format_address(&ctx.from),
                scope: "Cascading authorization across depth 1, 2, and 3".to_string(),
            }],
            state_effects: std::vec![std::format!(
                "Transfer {} tokens through intermediate relay contract",
                amount
            )],
            risk_level: "medium".to_string(),
            warnings: std::vec!["MULTI_CONTRACT_DELEGATION".to_string()],
        };

        let fix = build_fixture(
            &ctx.env,
            "03_three_level_chain",
            "3+ Level Sub-Invocation Chain",
            "Multi-contract call chain reaching depth 3 cascading authorization through intermediate contracts",
            &ctx.contract_id,
            "chain_call",
            args,
            &auths,
            events.events(),
            summary,
        );
        scenarios.push(("03_three_level_chain.json".to_string(), fix));
    }

    // 4. Dual depth auth
    {
        let ctx = setup_context();
        let sub_id = ctx.env.register(Forwarder, ());
        let value = 500_i128;
        ctx.client.dual_depth_auth(&ctx.from, &sub_id, &value);

        let auths = ctx.env.auths();
        let events = ctx.env.events().all();
        let args = std::vec![
            to_scval(&ctx.env, ctx.from.clone()),
            to_scval(&ctx.env, sub_id.clone()),
            to_scval(&ctx.env, value),
        ];

        let summary = ExpectedSummary {
            intent: "Authorize distinct operations at Depth 1 and Depth 2 with identical address"
                .to_string(),
            call_stack: std::vec![
                "Forwarder.dual_depth_auth(user, sub_contract, value)".to_string(),
                "Forwarder.inner_action(user, value)".to_string(),
            ],
            authorizers: std::vec![AuthorizerSummary {
                address: format_address(&ctx.from),
                scope: "Authorizes root call at Depth 1 and inner_action at Depth 2".to_string(),
            }],
            state_effects: std::vec![
                "Publish InnerEvent containing user address and value 500".to_string()
            ],
            risk_level: "low".to_string(),
            warnings: std::vec![],
        };

        let fix = build_fixture(
            &ctx.env,
            "04_dual_depth_auth",
            "Same Address at Multiple Depths",
            "The same address authorizing at two distinct levels of the invocation tree",
            &ctx.contract_id,
            "dual_depth_auth",
            args,
            &auths,
            events.events(),
            summary,
        );
        scenarios.push(("04_dual_depth_auth.json".to_string(), fix));
    }

    // 5. Contract delegation (authorize_as_current_contract)
    {
        let ctx = setup_context();
        let relay_id = ctx.env.register(Forwarder, ());
        let amount = 150_i128;
        StellarAssetClient::new(&ctx.env, &ctx.token_addr).mint(&ctx.contract_id, &500);

        ctx.client
            .contract_transfer(&ctx.from, &relay_id, &ctx.token_addr, &ctx.to, &amount);

        let auths = ctx.env.auths();
        let events = ctx.env.events().all();
        let args = std::vec![
            to_scval(&ctx.env, ctx.from.clone()),
            to_scval(&ctx.env, relay_id.clone()),
            to_scval(&ctx.env, ctx.token_addr.clone()),
            to_scval(&ctx.env, ctx.to.clone()),
            to_scval(&ctx.env, amount),
        ];

        let summary = ExpectedSummary {
            intent: "Contract authorizes downstream operations on its own behalf via authorize_as_current_contract".to_string(),
            call_stack: std::vec![
                "Forwarder.contract_transfer(caller, relay, token, to, amount)".to_string(),
                "Relay.execute_transfer(spender, token, to, amount)".to_string(),
                "Token.transfer(spender, to, amount)".to_string(),
            ],
            authorizers: std::vec![
                AuthorizerSummary {
                    address: format_address(&ctx.from),
                    scope: "Caller authorizes root contract_transfer entry point only".to_string(),
                },
                AuthorizerSummary {
                    address: format_address(&ctx.contract_id),
                    scope: "Contract authorizes relay to spend contract tokens via authorize_as_current_contract".to_string(),
                },
            ],
            state_effects: std::vec![std::format!(
                "Debit {} tokens from Forwarder contract balance; credit recipient balance",
                amount
            )],
            risk_level: "review".to_string(),
            warnings: std::vec!["CONTRACT_AS_SPENDER".to_string()],
        };

        let fix = build_fixture(
            &ctx.env,
            "05_contract_delegation",
            "Contract-to-Contract Delegation",
            "Contract authorizes downstream calls on its own behalf using authorize_as_current_contract",
            &ctx.contract_id,
            "contract_transfer",
            args,
            &auths,
            events.events(),
            summary,
        );
        scenarios.push(("05_contract_delegation.json".to_string(), fix));
    }

    // 6. Custom Smart Account Auth
    {
        let ctx = setup_context();
        let custom_acc = ctx.env.register(CustomAccount, ());
        let amount = 200_i128;
        StellarAssetClient::new(&ctx.env, &ctx.token_addr).mint(&custom_acc, &600);

        ctx.client
            .forward(&custom_acc, &ctx.token_addr, &ctx.to, &amount);

        let auths = ctx.env.auths();
        let events = ctx.env.events().all();
        let args = std::vec![
            to_scval(&ctx.env, custom_acc.clone()),
            to_scval(&ctx.env, ctx.token_addr.clone()),
            to_scval(&ctx.env, ctx.to.clone()),
            to_scval(&ctx.env, amount),
        ];

        let summary = ExpectedSummary {
            intent:
                "Forward tokens authorized by a custom smart account contract implementing __check_auth"
                    .to_string(),
            call_stack: std::vec![
                "CustomAccount.__check_auth(signature_payload, signatures, auth_contexts)"
                    .to_string(),
                "Forwarder.forward(custom_account, token, to, amount)".to_string(),
                "Token.transfer(custom_account, to, amount)".to_string(),
            ],
            authorizers: std::vec![AuthorizerSummary {
                address: format_address(&custom_acc),
                scope:
                    "Custom smart account contract validating signature payload via __check_auth"
                        .to_string(),
            }],
            state_effects: std::vec![std::format!(
                "Debit {} tokens from custom account; credit {} to recipient",
                amount,
                amount
            )],
            risk_level: "low".to_string(),
            warnings: std::vec!["SMART_ACCOUNT_INVOCATION".to_string()],
        };

        let fix = build_fixture(
            &ctx.env,
            "06_custom_smart_account_auth",
            "Custom Smart Account Authorization",
            "Smart contract account implementing CustomAccountInterface (__check_auth) validating invocations",
            &ctx.contract_id,
            "forward",
            args,
            &auths,
            events.events(),
            summary,
        );
        scenarios.push(("06_custom_smart_account_auth.json".to_string(), fix));
    }

    // 7. Forward with Meta and Mode
    {
        let ctx = setup_context();
        let meta = Meta {
            memo: SorobanString::from_str(&ctx.env, "invoice #42"),
            tags: vec![&ctx.env, symbol_short!("rent"), symbol_short!("monthly")],
        };
        let mode = Mode::Fast;
        let amount = 100_i128;

        ctx.client
            .forward_with_meta(&ctx.from, &ctx.token_addr, &ctx.to, &amount, &meta, &mode);

        let auths = ctx.env.auths();
        let events = ctx.env.events().all();
        let args = std::vec![
            to_scval(&ctx.env, ctx.from.clone()),
            to_scval(&ctx.env, ctx.token_addr.clone()),
            to_scval(&ctx.env, ctx.to.clone()),
            to_scval(&ctx.env, amount),
            to_scval(&ctx.env, meta),
            to_scval(&ctx.env, mode),
        ];

        let summary = ExpectedSummary {
            intent: std::format!(
                "Forward {} tokens attaching rich metadata struct and execution mode enum",
                amount
            ),
            call_stack: std::vec![
                "Forwarder.forward_with_meta(from, token, to, amount, meta, mode)".to_string(),
                "Token.transfer(from, to, amount)".to_string(),
            ],
            authorizers: std::vec![AuthorizerSummary {
                address: format_address(&ctx.from),
                scope: "Top-level invocation with custom types and downstream transfer".to_string(),
            }],
            state_effects: std::vec![std::format!(
                "Transfer {} tokens from sender to recipient with metadata memo and tags",
                amount
            )],
            risk_level: "low".to_string(),
            warnings: std::vec![],
        };

        let fix = build_fixture(
            &ctx.env,
            "07_forward_with_meta",
            "Forwarding with Custom Types (Struct & Enum)",
            "Forwards tokens carrying user-defined types (UDTs): a Meta struct and a Mode enum",
            &ctx.contract_id,
            "forward_with_meta",
            args,
            &auths,
            events.events(),
            summary,
        );
        scenarios.push(("07_forward_with_meta.json".to_string(), fix));
    }

    // Build Manifest with SHA-256 for each file
    let mut manifest_files = BTreeMap::new();
    for (filename, fix) in &scenarios {
        let json_bytes = serde_json::to_string_pretty(fix).expect("serialization succeeds") + "\n";
        let hash = Sha256::digest(json_bytes.as_bytes());
        let sha256_hex = hex::encode(hash);

        manifest_files.insert(
            filename.clone(),
            ManifestFileEntry {
                sha256: sha256_hex,
                scenario_id: fix.scenario_id.clone(),
                description: fix.name.clone(),
            },
        );
    }

    let manifest = Manifest {
        schema_version: "1.0.0".to_string(),
        files: manifest_files,
    };

    (scenarios, manifest)
}

#[test]
fn generate_and_verify_reproducible_fixtures() {
    let out_dir = fixtures_dir();
    fs::create_dir_all(&out_dir).expect("fixtures dir created");

    // First generation run
    let (scenarios1, manifest1) = generate_all_fixtures();

    for (filename, fix) in &scenarios1 {
        let file_path = out_dir.join(filename);
        let json_str = serde_json::to_string_pretty(fix).expect("valid json") + "\n";
        fs::write(&file_path, json_str.as_bytes()).expect("write fixture file");
    }

    let manifest_path = out_dir.join("manifest.json");
    let manifest_str = serde_json::to_string_pretty(&manifest1).expect("valid manifest") + "\n";
    fs::write(&manifest_path, manifest_str.as_bytes()).expect("write manifest");

    // Second generation run to assert 100% reproducibility
    let (scenarios2, manifest2) = generate_all_fixtures();

    assert_eq!(
        manifest1, manifest2,
        "Manifest must be identical across multiple runs"
    );

    for (idx, (filename, fix2)) in scenarios2.iter().enumerate() {
        let (fn1, fix1) = &scenarios1[idx];
        assert_eq!(filename, fn1);
        assert_eq!(
            fix1, fix2,
            "Fixture {} must be byte-for-byte identical across runs",
            filename
        );

        let file_path = out_dir.join(filename);
        let disk_content = fs::read_to_string(&file_path).expect("read file");
        let expected_json = serde_json::to_string_pretty(fix2).expect("valid json") + "\n";
        assert_eq!(
            disk_content, expected_json,
            "Written file {} must match freshly generated fixture",
            filename
        );
    }
}
