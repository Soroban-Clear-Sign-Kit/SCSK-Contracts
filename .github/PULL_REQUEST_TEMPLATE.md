## Description

Provide a clear and concise summary of the changes made and the motivation behind them.

---

## Type of Change

- [ ] `feat`: A new contract capability or fixture scenario
- [ ] `fix`: A bug fix or correction
- [ ] `test`: Added or updated unit tests / snapshots
- [ ] `docs`: Documentation updates
- [ ] `chore`: Build tooling, dependencies, or repository files

---

## Related Issues / Discussions

Closes #

---

## Verification & Testing

Describe how the changes were tested and include command outputs:

- [ ] `cargo build --target wasm32v1-none --release` passes
- [ ] `cargo test` passes all unit tests
- [ ] `cargo clippy --all-targets -- -D warnings` reports no warnings
- [ ] `cargo fmt -- --check` has no formatting diffs

### Command Output Summary:
```text
(paste terminal output here)
```

---

## Checklist

- [ ] Follows the "one task = one branch = one PR" rule.
- [ ] Branch is branched from and targets `main`.
- [ ] Conventional Commit messages are used.
- [ ] No secrets, keys, or `.env` files committed.
- [ ] Contract public APIs and existing behaviors are preserved (or breaking changes approved).
