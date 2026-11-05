# SEP-41 Token Interface Compliance

This document details the compliance of `contracts/token` with the **SEP-41: Soroban Token Interface** standard and CAP-46-6. `contracts/token` serves as a faithful, fully-compliant reference implementation of a fungible token on Soroban.

---

## 1. Compliance Matrix

Every public function defined in the SEP-41 interface is implemented, enforced with appropriate authorization checks, emits spec-compliant event topics/payloads, and is verified by dedicated unit tests.

| SEP-41 Interface Function | Signature | Primary Unit Test | Edge Case & Error Tests |
| :--- | :--- | :--- | :--- |
| **`allowance`** | `allowance(env: Env, from: Address, spender: Address) -> i128` | `test::test_allowance` | `test::test_expired_allowance_read` |
| **`approve`** | `approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32)` | `test::test_approve` | `test::test_approve_expired_expiration_ledger`<br>`test::test_zero_amount_approve`<br>`test::test_negative_amount_approve` |
| **`balance`** | `balance(env: Env, id: Address) -> i128` | `test::test_balance` | `test::test_balance` (non-existent address returns 0) |
| **`transfer`** | `transfer(env: Env, from: Address, to: Address, amount: i128)` | `test::test_transfer` | `test::test_self_transfer`<br>`test::test_self_transfer_insufficient_balance`<br>`test::test_zero_amount_transfer`<br>`test::test_transfer_above_balance`<br>`test::test_negative_amount_transfer` |
| **`transfer_from`** | `transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128)` | `test::test_transfer_from` | `test::test_self_transfer_from`<br>`test::test_expired_allowance_transfer_from`<br>`test::test_zero_amount_transfer_from`<br>`test::test_transfer_from_above_allowance`<br>`test::test_transfer_from_above_balance`<br>`test::test_negative_amount_transfer_from` |
| **`burn`** | `burn(env: Env, from: Address, amount: i128)` | `test::test_burn` | `test::test_zero_amount_burn`<br>`test::test_burn_above_balance`<br>`test::test_negative_amount_burn` |
| **`burn_from`** | `burn_from(env: Env, spender: Address, from: Address, amount: i128)` | `test::test_burn_from` | `test::test_expired_allowance_burn_from`<br>`test::test_zero_amount_burn_from`<br>`test::test_burn_from_above_balance`<br>`test::test_negative_amount_burn_from` |
| **`decimals`** | `decimals(env: Env) -> u32` | `test::test_decimals` | Verified initialization requirement |
| **`name`** | `name(env: Env) -> String` | `test::test_name` | Verified initialization requirement |
| **`symbol`** | `symbol(env: Env) -> String` | `test::test_symbol` | Verified initialization requirement |

### Additional Administration Functions

| Function | Signature | Primary Unit Test | Edge Case Tests |
| :--- | :--- | :--- | :--- |
| **`initialize`** | `initialize(env: Env, admin: Address, decimals: u32, name: String, symbol: String)` | `test::setup` | `test::test_initialize_twice` |
| **`mint`** | `mint(env: Env, to: Address, amount: i128)` | `test::test_token_flow` | `test::test_mint_without_admin_auth`<br>`test::test_negative_amount_mint` |
| **`clawback`** | `clawback(env: Env, from: Address, amount: i128)` | `test::test_token_flow` | `test::test_negative_amount_clawback` |
| **`set_admin`** | `set_admin(env: Env, new_admin: Address)` | `test::test_set_admin` | Requires current admin authorization |
| **`set_authorized`** | `set_authorized(env: Env, id: Address, authorize: bool)` | `test::test_set_authorized` | Requires admin authorization |

---

## 2. Event Topics and Data Payload Formats

All token lifecycle events adhere strictly to standard `soroban-token-sdk` definitions and SEP-41 specifications:

### `Approve`
- **Topics**: `["approve", from: Address, spender: Address]`
- **Data Payload**: `[amount: i128, expiration_ledger: u32]` (Vec format)
- **Emitted When**: `approve` is called to set, update, or clear an allowance.

### `Transfer` (`TransferWithAmountOnly`)
- **Topics**: `["transfer", from: Address, to: Address]`
- **Data Payload**: `amount: i128` (single-value format)
- **Emitted When**: Tokens move via `transfer` or `transfer_from`.

### `Burn`
- **Topics**: `["burn", from: Address]`
- **Data Payload**: `amount: i128` (single-value format)
- **Emitted When**: Tokens are destroyed via `burn` or `burn_from`.

### `Mint` (`MintWithAmountOnly`)
- **Topics**: `["mint", to: Address]`
- **Data Payload**: `amount: i128` (single-value format)
- **Emitted When**: New tokens are created via `mint`.

### `Clawback`
- **Topics**: `["clawback", from: Address]`
- **Data Payload**: `amount: i128` (single-value format)
- **Emitted When**: Tokens are reclaimed by the administrator.

---

## 3. Storage Architecture and TTL Extension

Soroban state archival rules require contracts to periodically extend the Time-To-Live (TTL) of storage entries. `contracts/token` implements automatic TTL extension on instance and persistent storage:

### Configuration Constants
```rust
const DAY_IN_LEDGERS: u32 = 17_280; // ~5 seconds per ledger close

// Instance storage (Admin, Decimals, Name, Symbol)
const INSTANCE_BUMP_AMOUNT: u32 = 7 * DAY_IN_LEDGERS; // 7 days
const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS; // 6 days

// Persistent storage (Balances, Allowances, Authorization State)
const BALANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS; // 30 days
const BALANCE_LIFETIME_THRESHOLD: u32 = BALANCE_BUMP_AMOUNT - DAY_IN_LEDGERS; // 29 days
```

### Automatic Extension Behavior
1. **Instance Storage**: Every state read or write invokes `extend_instance_ttl`, ensuring active token instances do not expire.
2. **Persistent Storage**:
   - `read_balance` and `write_balance` extend the TTL of the `DataKey::Balance(id)` entry.
   - `read_allowance` and `write_allowance` extend the TTL of valid `DataKey::Allowance(...)` entries.
   - `set_authorized` extends the TTL of `DataKey::State(id)`.
3. Verified in test: `test::test_ttl_extension_instance_and_persistent`.

---

## 4. Edge Cases & Spec Requirements

### Expired Allowances
- When `env.ledger().sequence() > expiration_ledger`:
  - `allowance(from, spender)` returns `0`.
  - `transfer_from` panics with `"expired allowance"`.
  - `burn_from` panics with `"expired allowance"`.
- Calling `approve` with `expiration_ledger < current_ledger` when `amount > 0` panics with `"expired allowance"`.

### Zero Amount
- Transfers, burns, and approvals with `amount == 0` succeed without altering balances.
- `approve(from, spender, 0, expiration_ledger)` successfully resets the allowance to 0.

### Insufficient Balance & Allowance
- Attempting to transfer or burn more than the account balance panics with `"insufficient balance"`.
- Attempting to spend more than the approved allowance panics with `"insufficient allowance"`.
- If an allowance is sufficient but the owner's balance is insufficient, `transfer_from` and `burn_from` panic with `"insufficient balance"`.

### Self-Transfer
- Transferring tokens where `from == to`:
  - Preserves the sender/recipient balance without duplicate deduction or state corruption.
  - Requires balance $\ge$ amount (fails with `"insufficient balance"` if balance is insufficient).
  - Emits the standard `Transfer` event.
  - Extends entry TTL in storage.

---

## 5. Property-Based Testing Invariants

In addition to hand-written unit tests, `contracts/token` is verified with property-based tests via `proptest` (configured for at least 256 test cases per property):

| Invariant Property | Test Name | Cases | Description |
| :--- | :--- | :--- | :--- |
| **Total Supply Conservation** | `prop_total_supply_conserved` | 256 | Sum of all account balances plus cumulative burned amount strictly equals initial minted supply across arbitrary action sequences. |
| **Non-Negative Balances** | `prop_balances_never_go_negative` | 256 | No sequence of operations (including negative amounts or amounts exceeding balance) can ever result in negative account balances. |
| **Non-Increasing Allowance** | `prop_allowance_never_increases_on_transfer_from` | 256 | Invoking `transfer_from` never increases allowance; on success it decrements by exact amount, and on failure allowance remains unchanged. |
| **Failed Call Atomicity** | `prop_failed_calls_leave_state_unchanged` | 256 | Any failed or reverted contract call leaves all balances and allowances strictly identical to pre-call snapshots. |

