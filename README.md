# Soroban Clear-Sign Kit - Smart Contracts (`SCSK-Contracts`)

This repository contains Soroban smart contracts used for testing, verifying, and benchmarking the **Soroban Clear-Sign Kit (SCSK)** decoding engine, authorization tree analyzers, and simulation diffing tools.

## Structure

```
.
├── Cargo.toml
├── contracts/
│   ├── fixture/          # Captures nested auth scenarios for Clear-Sign Kit testing
│   └── token/            # Minimal SEP-41 token implementation and mock USDC
└── README.md
```

## Prerequisites

- [Rust](https://www.rust-lang.org/) and `cargo` (v1.80+)
- `wasm32v1-none` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli) (v22+)

## Building Contracts

```bash
cargo build --target wasm32v1-none --release
```

## Testing Contracts

```bash
cargo test
```
