# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `feat(fixture)`: Covered common Soroban authorization shapes with contract entry points and exact `env.auths()` assertion tests:
  - `forward_for_args`: Custom argument authorization via `require_auth_for_args`.
  - `chain_call` and `chain_step`: 3+ level sub-invocation chain cascading across two Forwarder contracts and the Token contract.
  - `dual_depth_auth` and `inner_action`: Same address authorizing at two distinct tree depths (Depth 1 and Depth 2).
  - `contract_transfer` and `execute_transfer`: Contract-to-contract authorization using `env.authorize_as_current_contract`.
  - `CustomAccount` and `__check_auth`: Custom smart account contract implementing `CustomAccountInterface`.
- `docs(scenarios)`: Added `docs/SCENARIOS.md` documenting each authorization scenario, its purpose, entry points, and expected authorization tree structure.
- `chore(budget)`: Updated `size-budget.json` to accommodate new fixture scenarios within the 10% safety margin.
