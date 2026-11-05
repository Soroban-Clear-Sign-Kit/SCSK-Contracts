# Soroban Clear-Sign Kit - Smart Contracts (`SCSK-Contracts`)

This repository contains Soroban smart contracts used for testing, verifying, and benchmarking the **Soroban Clear-Sign Kit (SCSK)** decoding engine, authorization tree analyzers, and simulation diffing tools.

## Structure

```
.
├── Cargo.toml
├── contracts/
│   ├── fixture/          # Captures nested auth scenarios for Clear-Sign Kit testing
│   ├── risky/            # [TEST-ONLY] Benign contracts demonstrating risky patterns and bounds
│   ├── token/            # Minimal SEP-41 token implementation and mock USDC
│   └── values/           # Full ScVal and event type coverage testing contract
├── docs/                 # Specifications, risk patterns, and verifiable deployments
├── fixtures/             # Versioned test fixture suite and JSON scenarios
├── scripts/              # Size budget checks and deployment automation
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

## Deployments

All contracts are deployed and verified on Stellar Testnet. For deployed contract addresses, Wasm bytecode hashes, deployment dates, and read-only verification CLI commands, see:

👉 [**docs/DEPLOYMENTS.md**](docs/DEPLOYMENTS.md)

Deployments can be reproduced using `scripts/deploy-testnet.sh` with a testnet secret key supplied via environment variable:

```bash
export DEPLOYER_SECRET_KEY="S..."
export ADMIN_ADDRESS="G..."
./scripts/deploy-testnet.sh
```

