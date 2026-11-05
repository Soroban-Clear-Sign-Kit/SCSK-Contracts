#!/usr/bin/env bash
set -euo pipefail

# scripts/deploy-testnet.sh
# Deploys SCSK smart contracts to Stellar testnet using the Stellar CLI.
# The signing key is read strictly from an environment variable and NEVER hardcoded or committed.

DEPLOYER_SECRET="${DEPLOYER_SECRET_KEY:-${STELLAR_SECRET_KEY:-}}"
if [ -z "$DEPLOYER_SECRET" ]; then
  echo "Error: DEPLOYER_SECRET_KEY or STELLAR_SECRET_KEY environment variable must be set." >&2
  echo "Example usage:" >&2
  echo "  export DEPLOYER_SECRET_KEY=\"S...\"" >&2
  echo "  ./scripts/deploy-testnet.sh" >&2
  exit 1
fi

NETWORK="${STELLAR_NETWORK:-testnet}"
echo "==> Target network: $NETWORK"

# Determine admin address for initialization
ADMIN="${ADMIN_ADDRESS:-${DEPLOYER_PUBLIC_KEY:-}}"
if [ -z "$ADMIN" ]; then
  if [[ "$DEPLOYER_SECRET" =~ ^S ]]; then
    echo "Notice: DEPLOYER_SECRET is a secret key. If you want contracts initialized with an admin,"
    echo "set ADMIN_ADDRESS or DEPLOYER_PUBLIC_KEY in the environment."
  else
    ADMIN=$(stellar keys public-key "$DEPLOYER_SECRET" 2>/dev/null || true)
  fi
fi

# Step 1: Build release Wasm artifacts
echo "==> Building contracts in release mode..."
stellar contract build

WASM_DIR="target/wasm32v1-none/release"
TOKEN_WASM="$WASM_DIR/token.wasm"
FIXTURE_WASM="$WASM_DIR/fixture.wasm"
RISKY_WASM="$WASM_DIR/risky.wasm"
VALUES_WASM="$WASM_DIR/values.wasm"

for wasm in "$TOKEN_WASM" "$FIXTURE_WASM" "$RISKY_WASM" "$VALUES_WASM"; do
  if [ ! -f "$wasm" ]; then
    echo "Error: Build artifact not found at $wasm" >&2
    exit 1
  fi
done

# Step 2: Deploy contract instances
echo "==> Deploying token contract..."
TOKEN_ID=$(stellar contract deploy --wasm "$TOKEN_WASM" --source-account "$DEPLOYER_SECRET" --network "$NETWORK")
echo "  Token ID: $TOKEN_ID"

echo "==> Deploying fixture contract..."
FIXTURE_ID=$(stellar contract deploy --wasm "$FIXTURE_WASM" --source-account "$DEPLOYER_SECRET" --network "$NETWORK")
echo "  Fixture ID: $FIXTURE_ID"

echo "==> Deploying risky contract..."
RISKY_ID=$(stellar contract deploy --wasm "$RISKY_WASM" --source-account "$DEPLOYER_SECRET" --network "$NETWORK")
echo "  Risky ID: $RISKY_ID"

echo "==> Deploying values contract..."
VALUES_ID=$(stellar contract deploy --wasm "$VALUES_WASM" --source-account "$DEPLOYER_SECRET" --network "$NETWORK")
echo "  Values ID: $VALUES_ID"

# Step 3: Initialize contracts where required
if [ -n "$ADMIN" ]; then
  echo "==> Initializing token contract with admin: $ADMIN"
  stellar contract invoke \
    --id "$TOKEN_ID" \
    --source-account "$DEPLOYER_SECRET" \
    --network "$NETWORK" \
    -- initialize --admin "$ADMIN" --decimals 7 --name "Clear-Sign Kit Token" --symbol "SCSK" || true

  echo "==> Initializing risky contract with admin: $ADMIN"
  stellar contract invoke \
    --id "$RISKY_ID" \
    --source-account "$DEPLOYER_SECRET" \
    --network "$NETWORK" \
    -- init --admin "$ADMIN" || true
else
  echo "Notice: Admin address not specified; skipping initialize calls."
fi

echo ""
echo "================ Deployment Summary ================"
echo "Network:    $NETWORK"
echo "Date (UTC): $(date -u +'%Y-%m-%d %H:%M:%S UTC')"
echo "Token ID:   $TOKEN_ID"
echo "Fixture ID: $FIXTURE_ID"
echo "Risky ID:   $RISKY_ID"
echo "Values ID:  $VALUES_ID"
echo "====================================================="

