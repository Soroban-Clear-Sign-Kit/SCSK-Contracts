# Soroban Clear-Sign Kit - Smart Contracts (`SCSK-Contracts`)

This repository contains Soroban smart contracts used for testing, verifying, and benchmarking the **Soroban Clear-Sign Kit (SCSK)** decoding engine, authorization tree analyzers, and simulation diffing tools.

## Structure

```
.
├── Cargo.toml
├── contracts/
│   └── hello-world/      # Sample contract verifying basic invocation and return types
└── README.md
```

## Prerequisites

- [Rust](https://www.rust-lang.org/) and `cargo` (v1.80+)
- `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli) (v22+)

## Building Contracts

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Testing Contracts

```bash
cargo test
```
