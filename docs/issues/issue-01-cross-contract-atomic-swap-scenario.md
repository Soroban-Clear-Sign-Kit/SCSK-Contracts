# Issue 01: Multi-Party Atomic Token Swap Authorization Scenario (C3)

## Description
The test fixture contract (`contracts/fixture`) currently provides authorization patterns for single principals (`require_auth`), custom arguments (`require_auth_for_args`), multi-depth sub-invocations (`chain_call`), and smart account validation (`CustomAccount`). However, decentralized exchanges and escrow protocols commonly execute multi-party atomic swaps where **two distinct accounts** must authorize their respective legs of a trade within the same transaction.

Adding an atomic swap scenario will provide clear-sign decoders and hardware wallets with canonical test vectors for multi-principal authorization trees and opposing token transfers.

## Complexity
`Medium`

## Authorization & State Changes
- **Who authorizes**:
  - `user_a` must authorize the transfer of `amount_a` of `token_a` to `user_b` (`user_a.require_auth()`).
  - `user_b` must authorize the transfer of `amount_b` of `token_b` to `user_a` (`user_b.require_auth()`).
- **Contract execution**: The `Forwarder` contract verifies authorizations from both `user_a` and `user_b` and dispatches transfers against the respective token contracts.

## Requirements
1. Implement `swap` in `contracts/fixture/src/lib.rs`:
   ```rust
   pub fn swap(
       env: Env,
       user_a: Address,
       token_a: Address,
       amount_a: i128,
       user_b: Address,
       token_b: Address,
       amount_b: i128,
   )
   ```
2. The function must enforce authorization from both `user_a` and `user_b`, and transfer `token_a` from `user_a` to `user_b`, and `token_b` from `user_b` to `user_a`.
3. Add a unit test in `contracts/fixture/src/test.rs` verifying successful execution and checking recorded `env.auths()` containing two distinct root authorization trees.
4. Integrate Scenario 08 (`08_atomic_swap`) into `contracts/fixture/src/generator.rs` to produce `fixtures/08_atomic_swap.json` and update `fixtures/manifest.json`.
5. Document Scenario 08 in `docs/SCENARIOS.md` with authorization tree structure and purpose.
6. Verify size budget compliance (`python scripts/check-size-budget.py`), format (`cargo fmt`), clippy (`cargo clippy`), and build (`stellar contract build`).

## Files to Touch
- `contracts/fixture/src/lib.rs`
- `contracts/fixture/src/test.rs`
- `contracts/fixture/src/generator.rs`
- `docs/SCENARIOS.md`
- `fixtures/08_atomic_swap.json`
- `fixtures/manifest.json`
- `size-budget.json` (if WASM size requires minor adjustment)

## Suggested Execution
1. Add `swap` function to `Forwarder` in `contracts/fixture/src/lib.rs`. Call `user_a.require_auth()` and `user_b.require_auth()`, followed by invoking `token::Client::new(&env, &token_a).transfer(&user_a, &user_b, &amount_a)` and `token::Client::new(&env, &token_b).transfer(&user_b, &user_a, &amount_b)`.
2. Add unit test `test_swap_atomic_two_parties` in `contracts/fixture/src/test.rs` initializing two mock tokens and verifying that omitting either signature panics.
3. In `contracts/fixture/src/generator.rs`, register a second token contract and append `build_scenario_08_atomic_swap` to the scenario generation loop.
4. Run `cargo test --package fixture generate_and_verify_reproducible_fixtures` to update the fixture file and `manifest.json`.
5. Update `docs/SCENARIOS.md` with an invocation diagram and topic summary for Scenario 08.
6. Build release contracts via `stellar contract build` and execute `python scripts/check-size-budget.py`.

## Acceptance Criteria
- [ ] `swap` function is implemented in `contracts/fixture` and verifies authorizations from both parties.
- [ ] Unit tests assert successful execution with two valid signatures and panics if either authorization is missing.
- [ ] `fixtures/08_atomic_swap.json` is generated with valid XDR and indexed in `fixtures/manifest.json`.
- [ ] `docs/SCENARIOS.md` documents Scenario 08 with expected authorization entries.
- [ ] `cargo test` passes all tests across the workspace.
- [ ] `stellar contract build` succeeds and bytecode complies with size budgets.

