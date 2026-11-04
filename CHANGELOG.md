# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `feat(values)`: Implemented comprehensive Soroban value types and multi-topic events in `contracts/values`:
  - Functions accepting and returning nested `Vec`, nested `Map`, tuples, `Option`, enums with data, `Bytes`, `BytesN<32>`, `String`, `Symbol`, `Address`, `i128`, `u128`, `I256`, `U256`, `Duration`, and `Timepoint`.
  - Emitted multi-topic events (3 to 4 topics) for every value type call.
  - Comprehensive unit test suite with 18 tests asserting roundtrip values and emitted events.
- `docs(values)`: Added `docs/VALUES-TYPES.md` documenting type coverage matrix, ScVal discriminants, event topic structures, and test verifications.
- `chore(budget)`: Updated `size-budget.json` with a 9226-byte budget for `values.wasm`.
- `feat(token)`: Implemented full SEP-41 token compliance for `contracts/token`:
  - Adopted standard `soroban-token-sdk` event definitions and topics (`Approve` with Vec payload `[amount, expiration_ledger]`, `TransferWithAmountOnly`, `Burn`, `MintWithAmountOnly`, `Clawback`).
  - Added strict error handling for negative amounts and expired allowances (`"expired allowance"`).
  - Added automatic TTL extension for instance storage and persistent balance/allowance storage.
  - Implemented safe self-transfer handling and zero-amount edge cases.
  - Expanded unit test coverage with 40 tests covering all 10 SEP-41 interface functions and edge cases.
- `docs(compliance)`: Added `docs/SEP41-COMPLIANCE.md` mapping every SEP-41 interface function to its primary unit tests, edge case tests, event structures, and storage TTL strategies.
- `chore(budget)`: Updated `size-budget.json` to allocate an 8882-byte budget (+10% margin on 8074 bytes) for `token.wasm`.
- `feat(fixture)`: Covered common Soroban authorization shapes with contract entry points and exact `env.auths()` assertion tests:
  - `forward_for_args`: Custom argument authorization via `require_auth_for_args`.
  - `chain_call` and `chain_step`: 3+ level sub-invocation chain cascading across two Forwarder contracts and the Token contract.
  - `dual_depth_auth` and `inner_action`: Same address authorizing at two distinct tree depths (Depth 1 and Depth 2).
  - `contract_transfer` and `execute_transfer`: Contract-to-contract authorization using `env.authorize_as_current_contract`.
  - `CustomAccount` and `__check_auth`: Custom smart account contract implementing `CustomAccountInterface`.
- `docs(scenarios)`: Added `docs/SCENARIOS.md` documenting each authorization scenario, its purpose, entry points, and expected authorization tree structure.
- `chore(budget)`: Updated `size-budget.json` to accommodate new fixture scenarios within the 10% safety margin.
