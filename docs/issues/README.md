# Candidate Issue Backlog (C3–C9 Improvements)

This directory contains well-formed, unpublished candidate issues for future development waves. Each issue provides complete specifications, requirements, touched files, step-by-step suggested execution, and acceptance criteria so any first-time contributor can pick up and execute the issue without external guidance.

---

## Issue Catalog

| Issue | Target Domain | Category | Complexity | Authorization Role | Recommended Wave |
| :--- | :--- | :--- | :--- | :--- | :--- |
| [**Issue 01: Multi-Party Atomic Token Swap Scenario**](issue-01-cross-contract-atomic-swap-scenario.md) | `contracts/fixture`, `docs/SCENARIOS.md` | C3 (Authorization Scenarios) | `Medium` | Both `user_a` and `user_b` must independently authorize their respective transfer legs (`require_auth`). | **Wave 2 (Ecosystem Scenarios)** |
| [**Issue 02: Custom Contract Error Enums & Result Types**](issue-02-custom-error-enums-and-result-types.md) | `contracts/values`, `docs/VALUES-TYPES.md` | C4 (Value Types & Errors) | `Medium` | Pure/read view function; `caller.require_auth()` for guarded state mutation. | **Wave 1 (Type & Error Rigor)** |
| [**Issue 03: Fee-on-Transfer & Slippage Risk Patterns**](issue-03-fee-on-transfer-and-sandwich-risk-patterns.md) | `contracts/risky`, `docs/RISK-PATTERNS.md` | C5 (Risk Heuristics) | `Medium` | `sender.require_auth()` authorizes transfer and fee deduction. | **Wave 1 (Risk Analysis)** |
| [**Issue 04: Clawback & Deauthorization State Invariants**](issue-04-clawback-and-authorization-state-invariants.md) | `contracts/token`, `docs/SEP41-COMPLIANCE.md` | C6 (Property Invariants) | `Medium` | `admin` authorizes `clawback` and `set_authorized`; caller authorizes transfer attempts. | **Wave 2 (Formal Token Invariants)** |
| [**Issue 05: Fixture Integrity & Scenario Inspection CLI**](issue-05-fixture-cli-export-and-simulation-diff-harness.md) | `scripts/`, `fixtures/` | C7 (Fixture Tooling) | `Trivial` | Developer tooling (read-only; no contract state mutation). | **Wave 1 (Developer Tooling Quick Win)** |

---

## Wave Allocation Strategy

### Wave 1: Immediate Enhancements (Core Rigor & Developer Tooling)
- **Issue 05**: Fixture Integrity Verification and Scenario Inspection CLI (`Trivial`). Provides immediate value for non-Rust frontend and wallet developers.
- **Issue 02**: Custom Contract Error Enums and Result Types Coverage (`Medium`). Expands ScVal decode testing to include error discriminants and `Result` types.
- **Issue 03**: Fee-on-Transfer and Slippage Discrepancy Risk Patterns (`Medium`). Enhances transaction simulation diffing test cases.

### Wave 2: Advanced Protocol & Invariant Extensions
- **Issue 01**: Multi-Party Atomic Token Swap Authorization Scenario (`Medium`). Adds multi-principal authorization trees and opposing token transfers.
- **Issue 04**: Clawback and Deauthorization State Invariant Property Tests (`Medium`). Extends `proptest` coverage to administrative controls and access revocation.

---

## Authoring Standards for Issues in this Directory
All candidate issues must satisfy:
1. **Self-Contained Clarity**: A new or first-time contributor must be able to complete the task without asking questions.
2. **Explicit Authorization Roles**: For contract issues, explicitly state which principals must sign or authorize state changes (`require_auth()`).
3. **Mandatory Build & Test Steps**: Require both `cargo test` and `stellar contract build` verification.
4. **Actionable Checklist**: Include an acceptance criteria checklist with at least 2 distinct `- [ ]` checkboxes.
5. **Complexity Classification**: Explicitly label complexity as `Trivial`, `Medium`, or `High`.

