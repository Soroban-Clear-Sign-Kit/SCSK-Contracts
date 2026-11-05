# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `docs(architecture)`: Added comprehensive architecture documentation and contributor clean-clone walkthrough in `ARCHITECTURE.md`:
  - Detailed system objectives, contract architecture, and execution roles for `contracts/token`, `contracts/fixture`, `contracts/risky`, and `contracts/values`.
  - Documented fixture generation and integrity verification pipeline (`generator.rs`, JSON scenario outputs, and `manifest.json` SHA-256 checks).
  - Provided a step-by-step clean-clone walkthrough enabling new contributors to clone, build, test, check size budgets, and regenerate fixtures without external guidance.
  - Added repository project map and navigation table in `README.md`.
- `feat(deploy)`: Published verifiable Stellar testnet deployments and automation:
  - Deployed `token`, `values`, `risky`, and `fixture` contracts to Stellar Testnet.
  - Implemented `scripts/deploy-testnet.sh` reading signing keys strictly from environment variables (`DEPLOYER_SECRET_KEY` / `STELLAR_SECRET_KEY`).
  - Added `docs/DEPLOYMENTS.md` documenting live contract IDs, Wasm bytecode hashes, deployment dates, and CLI read-only verification queries.
  - Enforced strict repository secret detection ensuring 0 secret keys exist in the working tree or commit history.
- `feat(fixtures)`: Implemented deterministic test fixture generation suite in `contracts/fixture/src/generator.rs`:
  - Produces versioned, reproducible test data covering 7 authorization scenarios and user-defined types.
  - Generates unsigned Stellar transaction envelope XDR (base64), authorization credentials, expected contract event streams, and human-readable analysis summaries.
  - Computes exact SHA-256 integrity checksums for each fixture in `fixtures/manifest.json` (`schema_version: "1.0.0"`).
  - Added CI verification step in `.github/workflows/ci.yml` that regenerates fixtures and asserts zero git diff.
- `docs(fixtures)`: Added `fixtures/README.md` documenting fixture reproducibility, schema definitions for `manifest.json` and scenario files, and scenario catalog.
- `test(token)`: Added property-based test suite using `proptest` proving token invariants across 256 cases per property:
  - Total supply conservation across arbitrary `transfer`, `burn`, and `transfer_from` sequences.
  - Balances never go negative under arbitrary operations and amounts.
  - Allowance never increases on `transfer_from` calls.
  - Failed calls leave contract state completely unchanged (atomic rollback).
- `docs(token)`: Updated `docs/SEP41-COMPLIANCE.md` with property-based testing invariant matrix and verification details.
- `feat(risky)`: Implemented benign test-only contract `contracts/risky` demonstrating high-risk patterns and bounds testing:
  - Functions exercising unlimited approval (`i128::MAX`), fresh spender approvals, single-step admin transfers (`set_admin`), in-place contract bytecode upgrades (`update_current_contract_wasm`), very large transfers, and oversize/deeply-nested arguments for protocol bounds.
  - Multi-topic events emitted for every pattern invocation.
  - Complete unit test suite with 8 tests asserting outcomes, state updates, and event payloads.
- `docs(risky)`: Added `docs/RISK-PATTERNS.md` describing each pattern, why it is risky, SCSK detection heuristics, and recommended safe alternatives.
- `chore(budget)`: Updated `size-budget.json` with a 7016-byte budget for `risky.wasm`.
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
