# Deployments

This document lists the verifiable smart contract deployments for the Clear-Sign Kit contracts on Stellar Testnet.

## Deployed Contracts

| Contract | Network | Deploy Date | Contract ID | Wasm Hash |
| :--- | :--- | :--- | :--- | :--- |
| `token` | `testnet` | 2026-11-05 | `CBS3VZCJOUIL3XS7B5P2DAHKN27XUCFXJTCVPXOF3U5OXBQLJNJDBQDB` | `2a1a59c78029e11f7db568b2161424dda99f298049cd1a05c90f9329d7f3f21d` |
| `values` | `testnet` | 2026-11-05 | `CDCKMUH7PKJBTPWSYYO6PJTEGYIMF67KJY6XQN56NOSVUDWNK4TVRJH4` | `07ac3a93126419b5d5bbfb01b4eb6d00ed5d69909b1ee7635031a9f634cd4af1` |
| `risky` | `testnet` | 2026-11-05 | `CCEDMGONOW7NGP3BJ3CQXRPTWEFYWDCTVSHOCDOHQKUTZZLPNMEOBGH5` | `7f8443c5e6053695b957d8b7bcbbd4abef9d67139986ef330dddcfb59a13e506` |
| `fixture` | `testnet` | 2026-11-05 | `CCE4BZWIBMRGK5HPZMKN3BFOV4AKRG4PRYRTFMCDYQYWUXFSLOREBLIB` | `300ca0aa66cf4ef644d65745e9cf0bf7d24dc2232e561307e5774d0a8e92676a` |

---

## On-Chain Verification

Every listed contract ID can be independently verified on-chain via read-only inspection using the Stellar CLI.

### 1. Token Contract (`CBS3VZCJOUIL3XS7B5P2DAHKN27XUCFXJTCVPXOF3U5OXBQLJNJDBQDB`)

Verify the contract instance ledger entry and Wasm hash:
```bash
stellar contract read \
  --id CBS3VZCJOUIL3XS7B5P2DAHKN27XUCFXJTCVPXOF3U5OXBQLJNJDBQDB \
  --network testnet
```

Execute read-only view simulations (no transaction fee or signature required):
```bash
# Read token name ("SCSK Token")
stellar contract invoke \
  --id CBS3VZCJOUIL3XS7B5P2DAHKN27XUCFXJTCVPXOF3U5OXBQLJNJDBQDB \
  --network testnet \
  --source-account GC7MADZCPATFS2YPRDAXHNSO7ZEMKU6N4AIYILZEJ34UYTAPA3P2NKWX \
  --send=no \
  -- name

# Read token symbol ("SCSK")
stellar contract invoke \
  --id CBS3VZCJOUIL3XS7B5P2DAHKN27XUCFXJTCVPXOF3U5OXBQLJNJDBQDB \
  --network testnet \
  --source-account GC7MADZCPATFS2YPRDAXHNSO7ZEMKU6N4AIYILZEJ34UYTAPA3P2NKWX \
  --send=no \
  -- symbol

# Read token decimals (7)
stellar contract invoke \
  --id CBS3VZCJOUIL3XS7B5P2DAHKN27XUCFXJTCVPXOF3U5OXBQLJNJDBQDB \
  --network testnet \
  --source-account GC7MADZCPATFS2YPRDAXHNSO7ZEMKU6N4AIYILZEJ34UYTAPA3P2NKWX \
  --send=no \
  -- decimals
```

Stellar Explorer:
- [View on StellarExpert](https://stellar.expert/explorer/testnet/contract/CBS3VZCJOUIL3XS7B5P2DAHKN27XUCFXJTCVPXOF3U5OXBQLJNJDBQDB)
- [View on Stellar Lab](https://lab.stellar.org/r/testnet/contract/CBS3VZCJOUIL3XS7B5P2DAHKN27XUCFXJTCVPXOF3U5OXBQLJNJDBQDB)

---

### 2. Values Contract (`CDCKMUH7PKJBTPWSYYO6PJTEGYIMF67KJY6XQN56NOSVUDWNK4TVRJH4`)

Verify the contract instance ledger entry:
```bash
stellar contract read \
  --id CDCKMUH7PKJBTPWSYYO6PJTEGYIMF67KJY6XQN56NOSVUDWNK4TVRJH4 \
  --network testnet
```

Execute a read-only echo invocation simulating type roundtrips and event emission:
```bash
stellar contract invoke \
  --id CDCKMUH7PKJBTPWSYYO6PJTEGYIMF67KJY6XQN56NOSVUDWNK4TVRJH4 \
  --network testnet \
  --source-account GC7MADZCPATFS2YPRDAXHNSO7ZEMKU6N4AIYILZEJ34UYTAPA3P2NKWX \
  --send=no \
  -- echo_string --tag test --val hello
```

Stellar Explorer:
- [View on StellarExpert](https://stellar.expert/explorer/testnet/contract/CDCKMUH7PKJBTPWSYYO6PJTEGYIMF67KJY6XQN56NOSVUDWNK4TVRJH4)
- [View on Stellar Lab](https://lab.stellar.org/r/testnet/contract/CDCKMUH7PKJBTPWSYYO6PJTEGYIMF67KJY6XQN56NOSVUDWNK4TVRJH4)

---

### 3. Risky Contract (`CCEDMGONOW7NGP3BJ3CQXRPTWEFYWDCTVSHOCDOHQKUTZZLPNMEOBGH5`)

Verify the contract instance ledger entry:
```bash
stellar contract read \
  --id CCEDMGONOW7NGP3BJ3CQXRPTWEFYWDCTVSHOCDOHQKUTZZLPNMEOBGH5 \
  --network testnet
```

Execute a read-only query for the configured admin:
```bash
stellar contract invoke \
  --id CCEDMGONOW7NGP3BJ3CQXRPTWEFYWDCTVSHOCDOHQKUTZZLPNMEOBGH5 \
  --network testnet \
  --source-account GC7MADZCPATFS2YPRDAXHNSO7ZEMKU6N4AIYILZEJ34UYTAPA3P2NKWX \
  --send=no \
  -- get_admin
```

Stellar Explorer:
- [View on StellarExpert](https://stellar.expert/explorer/testnet/contract/CCEDMGONOW7NGP3BJ3CQXRPTWEFYWDCTVSHOCDOHQKUTZZLPNMEOBGH5)
- [View on Stellar Lab](https://lab.stellar.org/r/testnet/contract/CCEDMGONOW7NGP3BJ3CQXRPTWEFYWDCTVSHOCDOHQKUTZZLPNMEOBGH5)

---

### 4. Fixture Contract (`CCE4BZWIBMRGK5HPZMKN3BFOV4AKRG4PRYRTFMCDYQYWUXFSLOREBLIB`)

Verify the contract instance ledger entry:
```bash
stellar contract read \
  --id CCE4BZWIBMRGK5HPZMKN3BFOV4AKRG4PRYRTFMCDYQYWUXFSLOREBLIB \
  --network testnet
```

Stellar Explorer:
- [View on StellarExpert](https://stellar.expert/explorer/testnet/contract/CCE4BZWIBMRGK5HPZMKN3BFOV4AKRG4PRYRTFMCDYQYWUXFSLOREBLIB)
- [View on Stellar Lab](https://lab.stellar.org/r/testnet/contract/CCE4BZWIBMRGK5HPZMKN3BFOV4AKRG4PRYRTFMCDYQYWUXFSLOREBLIB)

---

## Deployment Process

Deployments are executed via [`scripts/deploy-testnet.sh`](../scripts/deploy-testnet.sh).

```bash
export DEPLOYER_SECRET_KEY="S..."
export ADMIN_ADDRESS="G..."
./scripts/deploy-testnet.sh
```

### Security Posture
- Secret signing keys are supplied **only** through environment variables (`DEPLOYER_SECRET_KEY` or `STELLAR_SECRET_KEY`).
- Secret keys are **never** committed into version control, hardcoded in scripts, or written to build artifacts.
- The repository enforces automated secret detection and history scanning in CI.

