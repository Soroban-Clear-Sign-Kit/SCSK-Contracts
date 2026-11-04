# Contributing

1. Install Rust 1.80+ and the `wasm32v1-none` target.
2. Make your change and add or update tests next to it.
3. Run the same checks as CI:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --locked -- -D warnings
   cargo test --locked
   cargo build --release --target wasm32v1-none
   ```

4. Commit the regenerated `test_snapshots` files that your change touches.
5. Keep commits small and use conventional prefixes (`feat`, `fix`, `test`, `docs`, `refactor`, `ci`).
