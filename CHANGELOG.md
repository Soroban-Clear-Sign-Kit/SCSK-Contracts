# Changelog

## Unreleased

### Token
- Enforce the `set_authorized` flag on `transfer`, `transfer_from` and `mint`.
- Treat expired allowances as zero and reject approvals with a past expiration ledger.
- Panic on balance overflow instead of wrapping.
- Reject `decimals` above 18 on `initialize`.
- Extend the TTL of balance and allowance entries on write.
- Add `admin()` and `authorized()` getters.

### Fixture
- Add `forward_many` to produce wide authorization trees.

### Tooling
- CI now runs `cargo fmt --check` and `cargo clippy -D warnings`, and caches cargo artifacts.
