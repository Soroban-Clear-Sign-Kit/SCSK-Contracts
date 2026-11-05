# Issue 04: Clawback and Deauthorization State Invariant Property Tests (C6)

## Description
The property-based testing suite in `contracts/token/tests/properties.rs` currently verifies four fundamental invariants across random sequences of transfers, burns, and allowances:
1. Total supply conservation.
2. Non-negative balances.
3. Monotonic decrease of allowance upon `transfer_from`.
4. Atomic rollback on failed calls.

The SEP-41 token specification also mandates compliance for regulatory and administrative controls, specifically `clawback` and account authorization toggling (`set_authorized`). Adding dedicated property-based tests for these functions ensures that:
- Clawback operations strictly decrease total supply by the exact revoked balance without leaking supply.
- Deauthorized accounts (`authorized == false`) cannot transfer, receive, burn, or spend tokens even if a prior non-zero allowance exists.

## Complexity
`Medium`

## Authorization & State Changes
- **Who authorizes**:
  - `admin` must authorize `clawback(admin, from, amount)` and `set_authorized(admin, id, authorize)`.
  - Account owner must authorize any subsequent transfer or burn attempts.
- **State execution**:
  - `clawback` decreases balance of `from` and decrements total supply.
  - `set_authorized(..., false)` locks the account state.

## Requirements
1. Add property test `prop_clawback_conserves_supply` in `contracts/token/tests/properties.rs`:
   - Configured with `proptest!` running at least 256 iterations.
   - Generates arbitrary initial balances and random clawback amounts.
   - Asserts: `balance_after == balance_before - clawback_amount` and `total_supply_after == total_supply_before - clawback_amount`.
2. Add property test `prop_deauthorized_account_isolation` in `contracts/token/tests/properties.rs`:
   - Configured with `proptest!` running at least 256 iterations.
   - Sets up accounts with initial balances and active allowances.
   - Invokes `set_authorized(admin, account, false)`.
   - Asserts that all attempted `transfer`, `transfer_from`, `burn`, and `burn_from` operations fail and leave state strictly unchanged.
3. Update `docs/SEP41-COMPLIANCE.md` under the Property-Based Invariant Verification section to document the two new invariants and their generation models.
4. Verify size budget compliance (`python scripts/check-size-budget.py`), format (`cargo fmt`), clippy (`cargo clippy`), and build (`stellar contract build`).

## Files to Touch
- `contracts/token/tests/properties.rs`
- `docs/SEP41-COMPLIANCE.md`

## Suggested Execution
1. In `contracts/token/tests/properties.rs`, import `clawback` and `set_authorized` from `token::Client`.
2. Implement strategy generating arbitrary accounts, non-zero initial supply, and valid clawback amounts `1..=balance`.
3. Assert that clawbacks execute when authorized by admin, decrease supply accurately, and reject when amount exceeds balance.
4. Implement strategy for `set_authorized(..., false)` and assert panic when attempting `transfer` or `transfer_from`.
5. Run `cargo test --test properties` to ensure all properties pass across 256 cases within acceptable time limits.
6. Document invariants in `docs/SEP41-COMPLIANCE.md`.

## Acceptance Criteria
- [ ] `prop_clawback_conserves_supply` is implemented and verified over at least 256 cases.
- [ ] `prop_deauthorized_account_isolation` is implemented and verified over at least 256 cases.
- [ ] Any failed calls during property execution leave storage state completely untouched.
- [ ] `docs/SEP41-COMPLIANCE.md` documents the two new property tests and invariant guarantees.
- [ ] `cargo test` passes across all crates in CI time limits.
- [ ] `stellar contract build` succeeds.

