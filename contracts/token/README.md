# token

A minimal [SEP-41](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0041.md)
token used as a mock USDC when testing the Soroban Clear-Sign Kit.

## Entrypoints

| Function | Auth | Notes |
| --- | --- | --- |
| `initialize(admin, decimals, name, symbol)` | none | One-shot. `decimals` must be `<= 18`. |
| `transfer(from, to, amount)` | `from` | Both parties must be authorized. |
| `transfer_from(spender, from, to, amount)` | `spender` | Spends allowance, then balance. |
| `approve(from, spender, amount, expiration_ledger)` | `from` | Expiration may not be in the past (unless `amount` is `0`). |
| `mint(to, amount)` | admin | `to` must be authorized. |
| `burn(from, amount)` / `burn_from(spender, from, amount)` | `from` / `spender` | |
| `clawback(from, amount)` | admin | Works on de-authorized accounts. |
| `set_admin(new_admin)` | admin | |
| `set_authorized(id, authorize)` | admin | Accounts are authorized by default. |
| `balance`, `allowance`, `authorized`, `admin`, `decimals`, `name`, `symbol` | none | Read-only. |

## Behaviour worth knowing

- Amounts must be non-negative; negative values panic with `negative amount`.
- An allowance whose `expiration_ledger` has passed reads as `0`.
- Balance and allowance entries have their TTL extended whenever they are written.
- Events follow the SEP-41 shapes (`transfer`, `approve`, `mint`, `burn`, `clawback`, `set_admin`,
  `set_authorized`).
