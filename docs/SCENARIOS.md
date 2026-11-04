# Soroban Authorization Scenarios

This document details the authorization patterns and shapes covered by the test fixture contracts (`contracts/fixture`) in SCSK-Contracts. Each scenario illustrates how Soroban records and evaluates authorization trees (`env.auths()`) across contracts, custom arguments, multi-level call stacks, and custom accounts.

---

## Overview of Authorization Shapes

| Scenario | Contract Entry Point | Authorizing Principal | Depth | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Standard `require_auth`** | `forward` | User Account (`from`) | 2 | Top-level function authorization requiring matching arguments and nested token transfer. |
| **Custom Args `require_auth_for_args`** | `forward_for_args` | User Account (`from`) | 2 | Explicit authorization over custom arguments decoupled from the top-level method signature. |
| **3+ Level Sub-Invocation Chain** | `chain_call` &rarr; `chain_step` | User Account (`from`) | 3 | Authorization cascading through a 3-contract stack (Forwarder &rarr; Relay &rarr; Token). |
| **Same Address at Multiple Depths** | `dual_depth_auth` &rarr; `inner_action` | User Account (`user`) | 2 | The same principal authorizing at both Depth 1 (root call) and Depth 2 (nested sub-invocation). |
| **Contract-to-Contract Delegation** | `contract_transfer` &rarr; `execute_transfer` | Caller Account & Contract | 1 (User), In-VM (Contract) | Contract authorizes downstream contract-to-contract operations via `authorize_as_current_contract`. |
| **Custom Smart Account Auth** | `forward` via `CustomAccount` | Custom Account Contract | 2 | Custom smart account validating invocations via the standard `__check_auth` interface. |

---

## 1. Standard Authorization: `require_auth`

### Description
Demonstrates the baseline Soroban authorization pattern where a user authorizes a contract function and all sub-invocations triggered by it. `from.require_auth()` checks that the caller authorized the `forward` call with identical arguments, and transitively authorizes the downstream `token.transfer` call.

### Entry Point
```rust
pub fn forward(env: Env, from: Address, token: Address, to: Address, amount: i128)
```

### Expected Authorization Tree (`env.auths()`)
```text
from: Address
└── forward(from, token, to, amount)
    └── transfer(from, to, amount)
```

### Tree Representation
```rust
vec![(
    from.clone(),
    AuthorizedInvocation {
        function: AuthorizedFunction::Contract((
            contract_id.clone(),
            symbol_short!("forward"),
            (from.clone(), token.clone(), to.clone(), amount).into_val(&env),
        )),
        sub_invocations: std::vec![AuthorizedInvocation {
            function: AuthorizedFunction::Contract((
                token.clone(),
                symbol_short!("transfer"),
                (from.clone(), to.clone(), amount).into_val(&env),
            )),
            sub_invocations: std::vec![],
        }],
    },
)]
```

---

## 2. Custom Arguments Authorization: `require_auth_for_args`

### Description
Demonstrates decoupled authorization where the arguments checked during authorization differ from the top-level function arguments. The contract invokes:
```rust
from.require_auth_for_args(
    (symbol_short!("forw_args"), token.clone(), to.clone(), amount).into_val(&env),
);
```
This allows authorizing custom payloads, permissions, or domain-specific descriptors while preserving the public API of the contract.

### Entry Point
```rust
pub fn forward_for_args(env: Env, from: Address, token: Address, to: Address, amount: i128)
```

### Expected Authorization Tree (`env.auths()`)
```text
from: Address
└── forward_for_args(Symbol("forw_args"), token, to, amount)
    └── transfer(from, to, amount)
```

### Tree Representation
```rust
vec![(
    from.clone(),
    AuthorizedInvocation {
        function: AuthorizedFunction::Contract((
            contract_id.clone(),
            Symbol::new(&env, "forward_for_args"),
            (
                symbol_short!("forw_args"),
                token.clone(),
                to.clone(),
                amount,
            ).into_val(&env),
        )),
        sub_invocations: std::vec![AuthorizedInvocation {
            function: AuthorizedFunction::Contract((
                token.clone(),
                symbol_short!("transfer"),
                (from.clone(), to.clone(), amount).into_val(&env),
            )),
            sub_invocations: std::vec![],
        }],
    },
)]
```

---

## 3. 3+ Level Sub-Invocation Chain Across Multiple Contracts

### Description
Exercises authorization cascading across multiple independent contracts reaching a depth of 3 or more:
1. **Level 1**: Top-level contract `Forwarder` executes `chain_call`.
2. **Level 2**: `Forwarder` invokes intermediate contract `relay` (`chain_step`), which checks `from.require_auth()`.
3. **Level 3**: Intermediate contract invokes `token` (`transfer`), which checks `from.require_auth()`.

### Entry Points
- `chain_call(env, from, relay, token, to, amount)` on root Forwarder
- `chain_step(env, from, token, to, amount)` on relay Forwarder

### Expected Authorization Tree (`env.auths()`)
```text
from: Address
└── chain_call(from, relay, token, to, amount)        [Depth 1: Root Forwarder]
    └── chain_step(from, token, to, amount)           [Depth 2: Relay Forwarder]
        └── transfer(from, to, amount)                [Depth 3: Token Contract]
```

### Tree Representation
```rust
vec![(
    from.clone(),
    AuthorizedInvocation {
        function: AuthorizedFunction::Contract((
            root_contract_id.clone(),
            Symbol::new(&env, "chain_call"),
            (from.clone(), relay_id.clone(), token.clone(), to.clone(), amount).into_val(&env),
        )),
        sub_invocations: std::vec![AuthorizedInvocation {
            function: AuthorizedFunction::Contract((
                relay_id.clone(),
                Symbol::new(&env, "chain_step"),
                (from.clone(), token.clone(), to.clone(), amount).into_val(&env),
            )),
            sub_invocations: std::vec![AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.clone(),
                    symbol_short!("transfer"),
                    (from.clone(), to.clone(), amount).into_val(&env),
                )),
                sub_invocations: std::vec![],
            }],
        }],
    },
)]
```

---

## 4. Same Address Authorizing at Multiple Tree Depths

### Description
Tests a scenario where a single address (`user`) authorizes distinct operations at multiple levels of the invocation tree.
- At **Depth 1**, `dual_depth_auth` calls `user.require_auth()`.
- It then invokes `sub_contract.inner_action(&user, &value)`.
- At **Depth 2**, `inner_action` on the sub-contract also requires `user.require_auth()` before emitting an `InnerEvent`.

### Entry Points
- `dual_depth_auth(env, user, sub_contract, value)` on caller Forwarder
- `inner_action(env, user, value)` on callee Forwarder

### Expected Authorization Tree (`env.auths()`)
```text
user: Address
└── dual_depth_auth(user, sub_contract, value)        [Depth 1: User Authorized]
    └── inner_action(user, value)                     [Depth 2: User Authorized]
```

### Tree Representation
```rust
vec![(
    user.clone(),
    AuthorizedInvocation {
        function: AuthorizedFunction::Contract((
            contract_id.clone(),
            Symbol::new(&env, "dual_depth_auth"),
            (user.clone(), sub_id.clone(), value).into_val(&env),
        )),
        sub_invocations: std::vec![AuthorizedInvocation {
            function: AuthorizedFunction::Contract((
                sub_id.clone(),
                Symbol::new(&env, "inner_action"),
                (user.clone(), value).into_val(&env),
            )),
            sub_invocations: std::vec![],
        }],
    },
)]
```

---

## 5. Contract-to-Contract Delegation: `authorize_as_current_contract`

### Description
When a contract needs to transfer its own assets or authorize operations on other contracts without delegating caller credentials down the stack, it uses `env.authorize_as_current_contract(...)`.
- The user (`caller`) authorizes only the root entry point `contract_transfer`.
- The calling contract authorizes `relay.execute_transfer` to spend its own tokens by registering a `SubContractInvocation` via `env.authorize_as_current_contract`.
- As a result, the user's recorded auth tree contains no sub-invocations (`sub_invocations: []`), and the downstream token transfer executes with the contract's explicit in-VM authority.

### Entry Points
- `contract_transfer(env, caller, relay, token, to, amount)` on root Forwarder
- `execute_transfer(env, spender, token, to, amount)` on relay Forwarder

### Expected Authorization Tree (`env.auths()`)
```text
caller: Address
└── contract_transfer(caller, relay, token, to, amount)
    (sub_invocations: empty — transfer is authorized by Contract A via authorize_as_current_contract)
```

### Tree Representation
```rust
vec![(
    caller.clone(),
    AuthorizedInvocation {
        function: AuthorizedFunction::Contract((
            contract_id.clone(),
            Symbol::new(&env, "contract_transfer"),
            (caller.clone(), relay_id.clone(), token.clone(), to.clone(), amount).into_val(&env),
        )),
        sub_invocations: std::vec![],
    },
)]
```

---

## 6. Custom Smart Account Authorization: `__check_auth`

### Description
Implements the Soroban standard `CustomAccountInterface` (`__check_auth`) on `CustomAccount`. Smart accounts can validate complex multi-signature schemes, account recovery rules, or session policies.
When `forward` is called with the custom account address as `from`, the host invokes `__check_auth` on the custom account contract to authenticate the transaction before proceeding with downstream token transfers.

### Implementation Interface
```rust
pub trait CustomAccountInterface {
    type Signature;
    type Error: Into<soroban_sdk::Error>;

    fn __check_auth(
        env: Env,
        signature_payload: Hash<32>,
        signatures: Self::Signature,
        auth_contexts: Vec<Context>,
    ) -> Result<(), Self::Error>;
}
```

### Expected Authorization Tree (`env.auths()`)
```text
custom_account: Contract Address
└── forward(custom_account, token, to, amount)
    └── transfer(custom_account, to, amount)
```

### Tree Representation
```rust
vec![(
    custom_account.clone(),
    AuthorizedInvocation {
        function: AuthorizedFunction::Contract((
            contract_id.clone(),
            symbol_short!("forward"),
            (custom_account.clone(), token.clone(), to.clone(), amount).into_val(&env),
        )),
        sub_invocations: std::vec![AuthorizedInvocation {
            function: AuthorizedFunction::Contract((
                token.clone(),
                symbol_short!("transfer"),
                (custom_account.clone(), to.clone(), amount).into_val(&env),
            )),
            sub_invocations: std::vec![],
        }],
    },
)]
```
