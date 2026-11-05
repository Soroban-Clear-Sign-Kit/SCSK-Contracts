# Issue 02: Custom Contract Error Enums and Result Types Coverage (C4)

## Description
`contracts/values` comprehensively exercises all positive `ScVal` value types (integers, strings, symbols, tuples, nested maps/vecs, and multi-topic events). However, Soroban smart contracts also return structured, typed errors using the `#[contracterror]` attribute and Rust's `Result<T, E>` return type.

When simulating transactions, wallets and clear-sign decoders must distinguish between unhandled panics and intentional, structured contract errors with specific error codes. Adding explicit error types and fallible entry points to `contracts/values` will allow simulation diffing engines to test and assert contract error decoding.

## Complexity
`Medium`

## Authorization & State Changes
- **Who authorizes**:
  - For `check_value`: Read-only, no authorization required.
  - For `guarded_mutation`: `caller` must authorize the operation (`caller.require_auth()`).
- **State execution**: If an error is returned, no state changes or events must be committed.

## Requirements
1. Define a `#[contracterror]` enum in `contracts/values/src/lib.rs`:
   ```rust
   #[contracterror]
   #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
   #[repr(u32)]
   pub enum ValuesError {
       InvalidValue = 101,
       ThresholdExceeded = 102,
       OperationForbidden = 103,
   }
   ```
2. Add fallible contract functions returning `Result<T, ValuesError>`:
   - `check_value(env: Env, val: i128) -> Result<i128, ValuesError>`
   - `guarded_mutation(env: Env, caller: Address, amount: i128) -> Result<i128, ValuesError>`
3. Emit a multi-topic event `SuccessResultEvent` upon successful execution.
4. Add comprehensive unit tests in `contracts/values/src/test.rs` asserting:
   - Successful execution returns `Ok(val)` and emits expected event.
   - Values below or above limits return expected `Err(ValuesError::...)` variants.
   - Simulation leaves instance/persistent state unchanged on error returns.
5. Update `docs/VALUES-TYPES.md` with a new section documenting error enum discriminants and `Result<T, E>` ScVal representation.
6. Verify size budget compliance (`python scripts/check-size-budget.py`), format (`cargo fmt`), clippy (`cargo clippy`), and build (`stellar contract build`).

## Files to Touch
- `contracts/values/src/lib.rs`
- `contracts/values/src/test.rs`
- `docs/VALUES-TYPES.md`
- `size-budget.json` (if WASM size requires minor adjustment)

## Suggested Execution
1. In `contracts/values/src/lib.rs`, declare `ValuesError` using `#[contracterror]`.
2. Add `check_value` and `guarded_mutation` to `ValuesContract`. Ensure `guarded_mutation` calls `caller.require_auth()` before validation checks.
3. Write unit tests in `contracts/values/src/test.rs` verifying that `ValuesError::ThresholdExceeded` is returned when `val > 1000` and `ValuesError::InvalidValue` when `val < 0`.
4. Document the error discriminant schema in `docs/VALUES-TYPES.md`.
5. Run `cargo test` and `stellar contract build` to verify that release compilation succeeds within budget.

## Acceptance Criteria
- [ ] `ValuesError` enum is declared with `#[contracterror]` and distinct error codes.
- [ ] Fallible entry points return `Result<T, ValuesError>` and are accessible via the generated Soroban client.
- [ ] Unit tests assert both `Ok` values and distinct `Err(ValuesError)` variants.
- [ ] `docs/VALUES-TYPES.md` documents error discriminants and `Result` ScVal decoding.
- [ ] `cargo test` passes across all crates.
- [ ] `stellar contract build` succeeds and bytecode passes size budget checks.

