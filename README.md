# Soroban Clear-Sign Kit - Smart Contracts (`SCSK-Contracts`)

[![CI](https://github.com/Soroban-Clear-Sign-Kit/SCSK-Contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Soroban-Clear-Sign-Kit/SCSK-Contracts/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

`SCSK-Contracts` provides canonical Soroban smart contracts, authorization harnesses, risk-pattern baselines, and deterministically generated test fixtures for the **Soroban Clear-Sign Kit (SCSK)**.

These contracts and test vectors serve as the foundational reference suite used to benchmark and verify client-side transaction decoders, hardware wallet signers, authorization tree visualizers, and simulation diffing engines across the Stellar and Soroban ecosystems.

---

## Project Map & Documentation

```
.
├── ARCHITECTURE.md          # System architecture, contract purposes, and clean-clone walkthrough
├── CHANGELOG.md             # Project release and change history (Keep a Changelog)
├── CONTRIBUTING.md          # Contributor guidelines, development workflow, and PR rules
├── Cargo.toml               # Workspace configuration and dependencies
├── size-budget.json         # Per-contract bytecode size budgets and enforcement thresholds
├── contracts/
│   ├── fixture/             # Authorization harness, sub-invocation trees, and fixture generator
│   ├── risky/               # [TEST-ONLY] Benign contracts demonstrating risky patterns and bounds
│   ├── token/               # SEP-41 token reference implementation with invariant testing
│   └── values/              # ScVal value types and multi-topic event testing suite
├── docs/
│   ├── ARCHITECTURE.md      # System architecture & contributor walkthrough link
│   ├── DEPLOYMENTS.md       # Verifiable testnet deployment addresses, hashes, and CLI queries
│   ├── RISK-PATTERNS.md     # Risk pattern catalog, detection heuristics, and mitigation guidance
│   ├── SCENARIOS.md         # Authorization scenario catalog and invocation tree structures
│   ├── SEP41-COMPLIANCE.md  # SEP-41 interface compliance matrix and proptest invariants
│   └── VALUES-TYPES.md      # Soroban ScVal and multi-topic event coverage matrix
├── fixtures/
│   ├── README.md            # Fixture manifest schemas, scenario formats, and regeneration rules
│   ├── manifest.json        # Integrity manifest containing SHA-256 hashes of all fixtures
│   └── *.json               # Deterministic scenario fixtures (unsigned XDR, auths, events)
└── scripts/
    ├── check-size-budget.py # WASM bytecode size budget enforcement script
    └── deploy-testnet.sh    # Testnet deployment automation script
```

### Key Documentation Links

| Resource | Description |
| :--- | :--- |
| [**Architecture & Walkthrough**](ARCHITECTURE.md) | Comprehensive system architecture, contract purposes, fixture generation pipeline, and new contributor clean-clone walkthrough. |
| [**SEP-41 Token Compliance**](docs/SEP41-COMPLIANCE.md) | Interface mapping, property-based invariants, event formats, and TTL management for `contracts/token`. |
| [**Authorization Scenarios**](docs/SCENARIOS.md) | In-depth breakdown of standard, custom args, multi-level, and custom account auth scenarios in `contracts/fixture`. |
| [**Risk Patterns Catalog**](docs/RISK-PATTERNS.md) | Reference catalog of high-risk patterns, bounds testing, analyzer heuristics, and safe alternatives in `contracts/risky`. |
| [**Value Types & Events**](docs/VALUES-TYPES.md) | Full coverage matrix of all Soroban `ScVal` types, compound structures, and multi-topic event streams in `contracts/values`. |
| [**Testnet Deployments**](docs/DEPLOYMENTS.md) | Live testnet contract IDs, Wasm bytecode hashes, deployment dates, and read-only verification commands. |
| [**Test Fixtures Suite**](fixtures/README.md) | Schema specifications for `manifest.json` and scenario files, alongside reproducibility verification instructions. |
| [**Contributing Guide**](CONTRIBUTING.md) | Development standards, Git hygiene, PR requirements, and conventional commit guidelines. |
| [**Changelog**](CHANGELOG.md) | Complete historical record of additions, enhancements, and fixes in Keep a Changelog format. |

---

> [!WARNING]
> **TEST-ONLY CONTRACTS**:
> Contracts in [`contracts/risky/`](contracts/risky/) (and all associated fixtures) are strictly **TEST-ONLY**. They implement patterns intentionally designed to trigger analyzer warnings and wallet risk heuristics (such as unlimited allowances, single-step admin transfers, in-place bytecode upgrades, very large transfers, and protocol bounds stress tests). Never deploy or use these contracts in production environments. See [`docs/RISK-PATTERNS.md`](docs/RISK-PATTERNS.md) for full descriptions.

---

## Quickstart

### Prerequisites

- [Rust](https://www.rust-lang.org/) and `cargo` (v1.80+)
- `wasm32v1-none` target (`rustup target add wasm32v1-none`)
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli) (v22+)
- Python 3 (for size budget verification)

### Build Contracts

Compile all contracts to optimized Soroban WASM targets:
```bash
cargo build --target wasm32v1-none --release
```
Or build via the Stellar CLI:
```bash
stellar contract build
```

Verify that compiled bytecode complies with configured size limits:
```bash
python scripts/check-size-budget.py
```

### Run Tests

Execute unit tests, property tests, and scenario tests:
```bash
cargo test
```

### Regenerate & Verify Fixtures

Deterministically regenerate all scenario fixtures in `fixtures/` and assert zero git diff:
```bash
cargo test --package fixture generate_and_verify_reproducible_fixtures
git diff --exit-code fixtures/
```

For the complete clean-clone step-by-step contributor walkthrough, see [**ARCHITECTURE.md**](ARCHITECTURE.md#4-clean-clone-contributor-walkthrough).

---

## Deployments

All contracts are deployed and verified on Stellar Testnet. For deployed contract addresses, Wasm bytecode hashes, deployment dates, and read-only verification CLI commands, see:

👉 [**docs/DEPLOYMENTS.md**](docs/DEPLOYMENTS.md)

Deployments can be reproduced using [`scripts/deploy-testnet.sh`](scripts/deploy-testnet.sh) with a testnet secret key supplied via environment variable:

```bash
export DEPLOYER_SECRET_KEY="S..."
export ADMIN_ADDRESS="G..."
./scripts/deploy-testnet.sh
```

---

## License

This project is licensed under the Apache 2.0 License. See [LICENSE](LICENSE) for details.
