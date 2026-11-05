# Test Fixtures & Scenario Data

This directory contains versioned, reproducible test fixtures generated directly from the smart contract test host in [`contracts/fixture`](../contracts/fixture).

These fixtures serve as the canonical reference data for verifying Soroban Clear-Sign Kit (SCSK) decoders, authorization tree analyzers, risk heuristics, and simulation diffing engines across backend and frontend environments.

---

## Reproducibility & Regeneration

The fixtures in this directory are generated deterministically using the test generator in `contracts/fixture/src/generator.rs`. All transaction envelopes, authorization credentials, event streams, and cryptographic hashes are derived without relying on external network requests or random seeds.

To regenerate and verify all fixtures locally:

```bash
cargo test --package fixture generate_and_verify_reproducible_fixtures
```

In CI, any divergence between committed test data and freshly generated fixtures triggers an immediate build failure via:

```bash
cargo test --package fixture generate_and_verify_reproducible_fixtures
git diff --exit-code fixtures/
```

---

## Manifest Schema (`manifest.json`)

`fixtures/manifest.json` indexes every generated fixture file alongside its schema version, SHA-256 integrity hash, scenario identifier, and human-readable description:

```json
{
  "schema_version": "1.0.0",
  "files": {
    "01_standard_require_auth.json": {
      "sha256": "0e115f0dd431e6d9406658605549e661563843765c882d4555ba8bc1f89dd995",
      "scenario_id": "01_standard_require_auth",
      "description": "Standard Authorization: require_auth"
    },
    ...
  }
}
```

### Fields

| Field | Type | Description |
| :--- | :--- | :--- |
| `schema_version` | `string` | Semantic version of the fixture format specification (`1.0.0`). |
| `files` | `object` | Map of fixture file names to their metadata and checksums (ordered alphabetically). |
| `files[filename].sha256` | `string` | SHA-256 hexadecimal digest of the exact file contents (including trailing newline). |
| `files[filename].scenario_id` | `string` | Unique identifier matching `docs/SCENARIOS.md`. |
| `files[filename].description` | `string` | Short human-readable summary of the scenario. |

---

## Scenario File Schema (`<scenario_id>.json`)

Each scenario file contains an unsigned transaction envelope, authorization trees, contract events, and expected analysis summaries.

```json
{
  "scenario_id": "01_standard_require_auth",
  "name": "Standard Authorization: require_auth",
  "description": "Baseline Soroban authorization where caller authorizes top-level entry point and nested token transfer",
  "tx_envelope_xdr": "<base64 encoded Stellar TransactionEnvelope XDR>",
  "auth_entries": [
    {
      "address": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHK3M",
      "credentials_type": "address",
      "root_invocation": {
        "contract": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAK3IM",
        "function": "forward",
        "args": [ ... ],
        "sub_invocations": [
          {
            "contract": "CBEPDNVYXQGWB5YUBXKJWYJA7OXTZW5LFLNO5JRRGE6Z6C5OSUZPCCEL",
            "function": "transfer",
            "args": [ ... ],
            "sub_invocations": []
          }
        ]
      },
      "entry_xdr": "<base64 encoded SorobanAuthorizationEntry XDR>"
    }
  ],
  "expected_events": [
    {
      "contract": "48f1b6b8bc0d60f7140dd49b6120fbaf3cdbab2adaeea631313d9f0bae9532f1",
      "topics": [
        "transfer",
        "contract:0000000000000000000000000000000000000000000000000000000000000003",
        "contract:0000000000000000000000000000000000000000000000000000000000000004",
        "aaa:GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEGWF"
      ],
      "data": "250",
      "event_xdr": "<base64 encoded ContractEvent XDR>"
    }
  ],
  "expected_summary": {
    "intent": "Forward 250 tokens from caller to recipient via standard require_auth",
    "call_stack": [
      "Forwarder.forward(from, token, to, amount)",
      "Token.transfer(from, to, amount)"
    ],
    "authorizers": [
      {
        "address": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHK3M",
        "scope": "Top-level invocation and downstream token transfer"
      }
    ],
    "state_effects": [
      "Transfer 250 tokens from sender balance to recipient balance"
    ],
    "risk_level": "low",
    "warnings": []
  }
}
```

### Schema Properties

1. **`scenario_id`** (`string`): Unique machine-readable scenario identifier.
2. **`name`** (`string`): Human-readable scenario name.
3. **`description`** (`string`): Explanation of the pattern exercised.
4. **`tx_envelope_xdr`** (`string`): Standard base64-encoded Stellar `TransactionEnvelope` XDR containing an `InvokeHostFunctionOp` with empty signatures (unsigned).
5. **`auth_entries`** (`array`): List of Soroban authorization credentials:
   - `address` (`string`): The authorizing address.
   - `credentials_type` (`string`): Credentials type (`address` or `source_account`).
   - `root_invocation` (`object`): Hierarchical tree representing the authorized function, its arguments, and recursive `sub_invocations`.
   - `entry_xdr` (`string`): Base64-encoded `SorobanAuthorizationEntry` XDR.
6. **`expected_events`** (`array`): Contract events emitted during simulation/execution:
   - `contract` (`string`): Hexadecimal contract identifier.
   - `topics` (`array`): Decoded topic values (symbols, addresses, strings).
   - `data` (`any`): Decoded event data payload.
   - `event_xdr` (`string`): Base64-encoded `ContractEvent` XDR.
7. **`expected_summary`** (`object`): Expected clear-sign summary:
   - `intent` (`string`): High-level plain English summary of the operation.
   - `call_stack` (`array<string>`): Ordered call stack of contract functions invoked.
   - `authorizers` (`array`): Principals required to sign and their authorization scope.
   - `state_effects` (`array<string>`): Financial or state mutations produced.
   - `risk_level` (`string`): Heuristic risk tier (`low`, `medium`, `review`, `blocked`).
   - `warnings` (`array<string>`): Security or analyzer warning codes (e.g. `MULTI_CONTRACT_DELEGATION`, `CONTRACT_AS_SPENDER`, `SMART_ACCOUNT_INVOCATION`).

---

## Scenario Catalog

| File | Scenario ID | Name | Depth | Risk Level |
| :--- | :--- | :--- | :---: | :---: |
| `01_standard_require_auth.json` | `01_standard_require_auth` | Standard Authorization: `require_auth` | 2 | `low` |
| `02_custom_args_auth.json` | `02_custom_args_auth` | Custom Arguments Authorization: `require_auth_for_args` | 2 | `low` |
| `03_three_level_chain.json` | `03_three_level_chain` | 3+ Level Sub-Invocation Chain | 3 | `medium` |
| `04_dual_depth_auth.json` | `04_dual_depth_auth` | Same Address at Multiple Depths | 2 | `low` |
| `05_contract_delegation.json` | `05_contract_delegation` | Contract-to-Contract Delegation | 1 (User), In-VM (Contract) | `review` |
| `06_custom_smart_account_auth.json` | `06_custom_smart_account_auth` | Custom Smart Account Authorization | 2 | `low` |
| `07_forward_with_meta.json` | `07_forward_with_meta` | Forwarding with Custom Types (Struct & Enum) | 2 | `low` |

