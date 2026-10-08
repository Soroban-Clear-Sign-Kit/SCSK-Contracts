#!/bin/bash
set -e

echo "Generating and funding alice..."
stellar keys generate --fund alice --network testnet || echo "Alice already exists"
echo "Generating and funding bob..."
stellar keys generate --fund bob --network testnet || echo "Bob already exists"

ALICE_PUB=$(stellar keys address alice)
BOB_PUB=$(stellar keys address bob)
echo "Alice: $ALICE_PUB"
echo "Bob: $BOB_PUB"

echo "Deploying test asset SAC..."
TOKEN_ID=$(stellar contract asset deploy --asset "TST:$ALICE_PUB" --source-account alice --network testnet)
echo "Token ID: $TOKEN_ID"

echo "Building fixture contract..."
cargo build --target wasm32v1-none --release

echo "Deploying fixture contract..."
CONTRACT_ID=$(stellar contract deploy --wasm ../../target/wasm32v1-none/release/fixture.wasm --source-account alice --network testnet)
echo "Contract ID: $CONTRACT_ID"

echo "Writing testnet.json..."
cat <<EOF > ../testnet.json
{
  "alice": "$ALICE_PUB",
  "bob": "$BOB_PUB",
  "tokenId": "$TOKEN_ID",
  "contractId": "$CONTRACT_ID"
}
EOF
echo "Done."
