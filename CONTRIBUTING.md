# Contributing to SCSK-Contracts

Thank you for your interest in contributing to the **Soroban Clear-Sign Kit - Smart Contracts (`SCSK-Contracts`)** repository!

This repository contains Soroban smart contracts and fixtures used to test, verify, and benchmark the SCSK decoding engine, authorization tree analyzers, and simulation diffing tools.

---

## 1. Prerequisites and Setup

Before contributing, ensure your environment meets the prerequisites:

- **Rust and cargo**: Version 1.80+ ([Install Rust](https://www.rust-lang.org/tools/install))
- **WASM Target**: `wasm32v1-none`
  ```bash
  rustup target add wasm32v1-none
  ```
- **Stellar CLI**: Version 22+ ([Install Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/stellar-cli))
- **Node.js & pnpm** (for fixture capture/mutation scripts in `fixtures/scripts`): Node.js v18+ and pnpm v9+

---

## 2. Development Workflow & Commands

Always verify that your code compiles, passes tests, and satisfies linting before submitting a pull request:

### Build Contracts
Build optimized release WASM binaries for the Soroban target:
```bash
cargo build --target wasm32v1-none --release
```
Or build using Stellar CLI (which also optimizes and strips contract bytecode):
```bash
stellar contract build
```

### WASM Size Budgets
Soroban smart contracts are subject to on-chain size limits and resource fees. To ensure contracts remain lightweight and prevent regression bloat, this repository enforces a size budget for each compiled and optimized `.wasm` binary:

- **Budget Configuration**: Tracked in [`size-budget.json`](file:///size-budget.json), initialized at the baseline optimized size + 10% headroom.
- **Verification Script**: Run the verification script after building:
  ```bash
  python scripts/check-size-budget.py
  ```
- **CI Gate**: Continuous Integration checks the size of every compiled `.wasm` file on every pull request. If any contract exceeds its configured budget, CI fails. If a feature legitimately requires more bytecode, propose an explicit budget adjustment in your PR with rationale.

### Run Tests
Execute the unit test suite:
```bash
cargo test
```

### Lint with Clippy
Run Clippy to catch common mistakes and enforce idioms:
```bash
cargo clippy --all-targets -- -D warnings
```

### Check Formatting
Ensure all code matches Rust formatting conventions:
```bash
cargo fmt -- --check
```
To automatically apply formatting:
```bash
cargo fmt
```

---

## 3. Branching, Commits, and Pull Request Rules

We enforce a strict development discipline to keep history clear, reproducible, and verifiable:

### One Task = One Branch = One Pull Request
- Every task must be developed on its own dedicated branch branched from `main`.
- **Never push directly to `main`**.
- Branch naming convention:
  - `feat/<short-description>` for new features or capabilities
  - `fix/<short-description>` for bug fixes
  - `docs/<short-description>` for documentation changes
  - `chore/<short-description>` for maintenance, dependencies, or repository files

### Atomic Commits & Conventional Commits
- Break work into the smallest meaningful, testable changes.
- Use [Conventional Commits](https://www.conventionalcommits.org/) format:
  - `feat: add ...`
  - `fix: correct ...`
  - `test: add unit test for ...`
  - `docs: update ...`
  - `chore: update ...`
- Never combine unrelated changes into a single commit.
- Every functional change must include tests that fail without the change and pass with it.

### Scope & Security
- **No secrets**: Never commit private keys, secret seeds, `.env` files, or production credentials.
- **Testnet only**: All fixture interactions and deployments target Stellar Testnet.
- **Contract code boundary**: Do not modify public contract APIs or existing behavior unless specifically tasked. If a breaking change is required, open an issue or RFC first.

---

## 4. Pull Request Checklist

Before opening a pull request, ensure:
1. `cargo test` passes 100%.
2. `cargo build --target wasm32v1-none --release` compiles without errors.
3. `cargo clippy` and `cargo fmt -- --check` report no violations.
4. All new files are documented and covered by tests.
5. The PR description accurately describes the changes, rationale, and verification output.
