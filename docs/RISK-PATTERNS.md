# Soroban Risky Patterns & Bounds Test Fixtures

> [!WARNING]
> **TEST-ONLY CONTRACT FIXTURE**
> All patterns implemented in `contracts/risky` are benign test fixtures created exclusively to test, audit, and benchmark the **Soroban Clear-Sign Kit (SCSK)** decoding engine, authorization tree analyzers, and wallet warning heuristics.
> **DO NOT** deploy or copy these patterns into production smart contracts without appropriate architectural safeguards.

---

## 1. Overview

Decentralized applications and smart contracts frequently contain operations that, while technically valid under the protocol, expose signers to critical security risks. The `contracts/risky` package provides standard reference implementations of these operations so that clear-sign tools, wallet simulation diffs, and security linters can verify their warning and diagnostic triggers.

---

## 2. Pattern Catalog

### 2.1. Unlimited Allowance Approval (`i128::MAX`)
- **Function**: `unlimited_approve(env, token, from, spender, expiration_ledger)`
- **Category**: Authorization / Asset Depletion Risk
- **Severity**: **HIGH**

#### Description
The function approves maximum possible allowance (`i128::MAX`) to a designated spender address.

#### Why It Is Risky
- **Perpetual Drain Risk**: An unlimited allowance grants the spender address the unconditional right to transfer all current and future tokens belonging to `from` until the allowance expires or is manually revoked.
- **Protocol Vulnerability Spillover**: If the spender contract subsequently suffers an exploit or governance compromise, attackers can drain tokens directly from all accounts that granted infinite approval, even years after the user stopped interacting with the protocol.

#### Clear-Sign Kit Detection & Safe Alternative
- **Detection**: Flag any approval where `amount == i128::MAX` or `amount >= balance * 100` with an explicit `UNLIMITED_APPROVAL` warning.
- **Safe Alternative**: Use exact-amount approvals per interaction, or bounded allowances that expire quickly (`expiration_ledger`).

---

### 2.2. Approval to a Fresh / Unverified Spender
- **Function**: `approve_fresh_spender(env, token, from, fresh_spender, amount, expiration_ledger)`
- **Category**: Phishing / Drainer Heuristics
- **Severity**: **HIGH**

#### Description
Approves a token allowance to an address with zero prior transaction history on the network.

#### Why It Is Risky
- **Phishing & Drainers**: Malicious frontends and phishing campaigns frequently deploy disposable, brand-new contracts or EOAs to bypass static blacklists and trick users into approving allowances.
- **Lack of Provenance**: An unverified address has no verified contract source, audit trail, or known protocol association.

#### Clear-Sign Kit Detection & Safe Alternative
- **Detection**: Cross-reference the spender address against known protocol registries and account age heuristics; flag spenders created within recent ledgers or with zero prior interactions.
- **Safe Alternative**: Require explicit user confirmation and show verified domain bindings when approving allowances to unknown spenders.

---

### 2.3. Single-Step Administrative Transfer (`set_admin`)
- **Function**: `set_admin(env, new_admin)`
- **Category**: Governance / Access Control Risk
- **Severity**: **CRITICAL**

#### Description
Immediately overwrites the contract's administrator address with `new_admin` in a single step upon `admin.require_auth()`.

#### Why It Is Risky
- **Irreversible Lockout**: If the administrator passes an incorrect address (e.g., typo, unowned account, or misconfigured multisig), administrative control is permanently and irreversibly lost. The contract becomes unmanageable.
- **Attacker Hijack**: Compromise of the current admin key instantly hands over total control without a grace period or recourse.

#### Clear-Sign Kit Detection & Safe Alternative
- **Detection**: Highlight any single-step `set_admin` call as a critical governance change.
- **Safe Alternative**: Implement a two-step transfer pattern:
  1. `propose_admin(new_admin)` records `pending_admin`.
  2. `accept_admin()` requires authorization from `pending_admin` to prove possession and intent.

---

### 2.4. In-Place Bytecode Upgrade (`update_current_contract_wasm`)
- **Function**: `upgrade(env, new_wasm_hash)`
- **Category**: Code Mutation / Trust Assumption Risk
- **Severity**: **CRITICAL**

#### Description
Replaces the contract's executing WebAssembly binary using `env.deployer().update_current_contract_wasm(new_wasm_hash)`.

#### Why It Is Risky
- **Total Logic Mutation**: Contract upgrades can completely change execution logic, bypass security invariants, modify storage schemas, or introduce backdoors/drainers while retaining the existing contract address and balances.
- **Silent Degradation**: Users interacting with a familiar contract address may not realize that the underlying code has changed.

#### Clear-Sign Kit Detection & Safe Alternative
- **Detection**: Detect calls to `update_current_contract_wasm`; display the new WASM hash prominently and diff against verified source builds.
- **Safe Alternative**: Enforce multisig governance, timelocks (delaying upgrade application by days or weeks), and immutable core storage invariants.

---

### 2.5. Very Large Transfer
- **Function**: `large_transfer(env, token, from, to, amount)`
- **Category**: Value Transfer / Treasury Depletion Risk
- **Severity**: **HIGH**

#### Description
Transfers an exceptionally large token balance (e.g., 100 quadrillion units or entire treasury balances).

#### Why It Is Risky
- **Treasury Draining**: Unchecked high-value transfers can exhaust protocol liquidity reserves or user balances in a single transaction.
- **Arithmetic & Decimal Confusion**: Users often misinterpret token decimals (e.g. 7 or 18 decimals) and accidentally authorize transfers several orders of magnitude higher than intended.

#### Clear-Sign Kit Detection & Safe Alternative
- **Detection**: Display human-readable token values formatted with correct decimals, fiat equivalents, and balance percentage warnings (e.g. "Transfers 100% of available balance").
- **Safe Alternative**: Implement withdrawal rate limits, daily transfer ceilings, and multi-party threshold approval for high-value transactions.

---

### 2.6. Oversize Argument Payload
- **Function**: `oversize_payload(env, payload: Bytes)`
- **Category**: Protocol Bounds / Denial of Service
- **Severity**: **MEDIUM**

#### Description
Passes a large binary payload (e.g., 32 KB) within protocol transaction and VM memory limits.

#### Why It Is Risky
- **Resource Exhaustion**: Large payloads consume significant CPU and VM memory during deserialization and hashing, increasing transaction fee costs and simulation failure risks.
- **Buffer & Storage Limits**: Excessive payloads can push transactions close to protocol ledger limits, resulting in unexpected transaction submission rejection.

#### Clear-Sign Kit Detection & Safe Alternative
- **Detection**: Measure and report transaction size and simulated resource consumption against network thresholds.
- **Safe Alternative**: Store large data off-chain (e.g. IPFS or Arweave) and pass cryptographic hashes (e.g. `BytesN<32>`) on-chain.

---

### 2.7. Deeply Nested Data Structures
- **Function**: `deeply_nested_structure(env, levels: Vec<Vec<Vec<Vec<u32>>>>)`
- **Category**: Protocol Bounds / Deserialization Depth
- **Severity**: **MEDIUM**

#### Description
Accepts a 4-level deeply nested collection hierarchy to exercise deserializer recursion and stack limits.

#### Why It Is Risky
- **Stack & Recursion Limits**: Deeply nested types test the limits of WASM recursion depth, potentially causing call stack overflows or high instruction consumption during XDR decoding.
- **Tooling Parse Failures**: Poorly implemented wallet parsers or indexers may crash or fail to clearly render deeply nested arguments to the signer.

#### Clear-Sign Kit Detection & Safe Alternative
- **Detection**: Flag nested type depth greater than 2-3 levels during ABI decoding.
- **Safe Alternative**: Flatten data structures using typed records with foreign keys or indexed identifiers instead of deeply nested collections.

---

## 3. Pattern Summary Matrix

| Pattern | Function Name | Risk Level | Event Emitted | Test Name |
| :--- | :--- | :--- | :--- | :--- |
| **Unlimited Approval** | `unlimited_approve` | HIGH | `unlimited_approve` | `test_unlimited_approve` |
| **Fresh Spender** | `approve_fresh_spender` | HIGH | `fresh_spender_approved` | `test_approve_fresh_spender` |
| **Single-Step Admin** | `set_admin` | CRITICAL | `admin_changed` | `test_set_admin_success` |
| **In-Place Upgrade** | `upgrade` | CRITICAL | `contract_upgraded` | `test_upgrade_contract` |
| **Large Transfer** | `large_transfer` | HIGH | `large_transfer_executed` | `test_large_transfer` |
| **Oversize Payload** | `oversize_payload` | MEDIUM | `oversize_payload_processed` | `test_oversize_payload` |
| **Deeply Nested** | `deeply_nested_structure` | MEDIUM | `deeply_nested_processed` | `test_deeply_nested_structure` |

---

## 4. Test Verification

All patterns are covered in `contracts/risky/src/test.rs` and assert:
- State transitions (admin changed, allowance updated, token balances updated).
- Exact event emissions (topics and payload).
- Invariant bounds within Soroban protocol limits.
