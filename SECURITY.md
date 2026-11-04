# Security Policy

## Scope: Test Fixtures and Verification Contracts

**Important Notice:** The smart contracts (`Forwarder`, `Token`) and scripts in this repository are designed exclusively as **test fixtures, benchmarks, and verification harnesses** for the **Soroban Clear-Sign Kit (SCSK)** decoding engine, authorization analyzers, and simulation diffing tools.

- These contracts are **not intended for production deployment** or managing mainnet user funds.
- Do not deploy or rely on these contracts on Stellar Mainnet.

---

## Supported Versions

Security updates and vulnerability remediation apply exclusively to the latest revision of the `main` branch:

| Branch / Version | Supported |
| :--- | :--- |
| `main` | :white_check_mark: |
| Historical tags / older branches | :x: |

---

## Reporting a Vulnerability

We take the security of our verification toolchain seriously. If you discover a security vulnerability or sensitive bug in the contracts, fixture generators, or decoding behaviors:

### Private Reporting Channel
- **Please DO NOT report security vulnerabilities via public GitHub issues, discussions, or pull requests.**
- Use **[GitHub Private Vulnerability Reporting](https://github.com/Soroban-Clear-Sign-Kit/SCSK-Contracts/security/advisories/new)** to submit your report confidentially.
- Alternatively, email the maintainers directly at: **security@clearsignkit.dev** with details and reproduction steps.

### What to Include
When submitting a report, please include:
1. Description of the issue and potential impact on SCSK clear-signing or decoding validation.
2. Step-by-step instructions or test cases reproducing the issue.
3. Any suggested mitigation or patch, if available.

### Disclosure Policy
We will acknowledge receipt of your report within 48 hours, validate the findings, and coordinate a fix prior to public disclosure.
