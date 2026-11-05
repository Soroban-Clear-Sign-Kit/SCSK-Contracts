# Soroban Clear-Sign Kit - Smart Contracts (`SCSK-Contracts`)

This repository contains Soroban smart contracts used for testing, verifying, and benchmarking the **Soroban Clear-Sign Kit (SCSK)** decoding engine, authorization tree analyzers, and simulation diffing tools.

## Structure

```
.
├── Cargo.toml
├── contracts/
│   ├── fixture/          # Captures nested auth scenarios for Clear-Sign Kit testing
│   ├── risky/            # [TEST-ONLY] Benign contracts demonstrating risky patterns and bounds
│   └── token/            # Minimal SEP-41 token implementation and mock USDC
└── README.md
```

> [!WARNING]
> **TEST-ONLY CONTRACTS**:
> Contracts in `contracts/risky/` (and all associated fixtures) are strictly **TEST-ONLY**. They implement patterns intentionally designed to trigger analyzer warnings and wallet risk heuristics (such as unlimited allowances, single-step admin transfers, in-place bytecode upgrades, very large transfers, and protocol bounds stress tests). Never deploy or use these contracts in production environments. See [`docs/RISK-PATTERNS.md`](docs/RISK-PATTERNS.md) for full descriptions.

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
