# Issue 03: Fee-on-Transfer and Slippage Discrepancy Risk Patterns (C5)

## Description
`contracts/risky` currently demonstrates high-risk vectors such as unlimited approvals, fresh spenders, single-step admin transfers, in-place contract upgrades, and oversized structures. 

Another prevalent source of user loss and transaction confusion in decentralized finance is **Fee-on-Transfer (Deflationary Tokens)** and **Unbounded Slippage**. In a fee-on-transfer model, the sender specifies an amount `X`, but the recipient only receives `X - fee`, with the fee burned or sent to a third party. If a wallet's clear-sign summary displays "Transfer 100 TOKEN" without indicating the deduction, users may sign unfavorable transactions or fail downstream contract expectations.

Adding a benign, test-only fee-on-transfer pattern will allow wallet simulation diffing engines and transaction analyzers to test detection of discrepancy between authorized amount and credited amount.

## Complexity
`Medium`

## Authorization & State Changes
- **Who authorizes**:
  - `sender` must authorize the transfer (`sender.require_auth()`).
- **State execution**: Deducts `amount` from `sender`, credits `amount - fee` to `recipient`, and credits `fee` to a designated treasury address or burns it.

## Requirements
1. Clearly mark all new types and functions with `[TEST-ONLY]` annotations in comments and docstrings.
2. Implement `fee_on_transfer` in `contracts/risky/src/lib.rs`:
   ```rust
   pub fn fee_on_transfer(
       env: Env,
       sender: Address,
       recipient: Address,
       amount: i128,
       fee_bps: u32,
   ) -> (i128, i128)
   ```
   where `fee_bps` is basis points (e.g. 500 = 5%). The return tuple is `(net_received, fee_deducted)`.
3. Emit a multi-topic event `FeeOnTransferEvent`:
   - Topics: `(symbol_short!("fee_tx"), sender.clone(), recipient.clone())`
   - Data: `(amount, net_received, fee_deducted)`
4. Add unit test in `contracts/risky/src/test.rs` asserting:
   - Calculation correctness: `fee = (amount * fee_bps as i128) / 10000`.
   - Event publication with exact topic structure.
   - Requirement of `sender` authorization.
5. Update `docs/RISK-PATTERNS.md` with:
   - Pattern description and why it is risky.
   - What SCSK analyzers and simulation diffing engines must detect (discrepancy between authorization parameter and balance delta).
   - Safe alternatives (explicit fee parameter in contract interface).
6. Verify size budget compliance (`python scripts/check-size-budget.py`), format (`cargo fmt`), clippy (`cargo clippy`), and build (`stellar contract build`).

## Files to Touch
- `contracts/risky/src/lib.rs`
- `contracts/risky/src/test.rs`
- `docs/RISK-PATTERNS.md`
- `size-budget.json` (if WASM size requires minor adjustment)

## Suggested Execution
1. In `contracts/risky/src/lib.rs`, define `FeeOnTransferEvent` with `#[contractevent]`.
2. Add `fee_on_transfer` to `RiskyContract`. Require `sender.require_auth()`, compute `fee`, emit the event, and return `(net_amount, fee)`.
3. In `contracts/risky/src/test.rs`, write `test_fee_on_transfer_deduction` testing a 1000-token transfer with a 10% (1000 bps) fee.
4. Add the pattern specification to `docs/RISK-PATTERNS.md`.
5. Run `cargo test` and `stellar contract build` to ensure the contract builds and remains within the size budget.

## Acceptance Criteria
- [ ] `fee_on_transfer` is implemented, labeled `[TEST-ONLY]`, and requires sender authorization.
- [ ] Multi-topic event `FeeOnTransferEvent` is published with sender, recipient, and amount breakdown.
- [ ] Unit tests assert correct mathematical deduction and authorization enforcement.
- [ ] `docs/RISK-PATTERNS.md` documents the pattern, analyzer detection heuristics, and mitigation.
- [ ] `cargo test` passes across all workspace crates.
- [ ] `stellar contract build` succeeds and bytecode passes size budget checks.
