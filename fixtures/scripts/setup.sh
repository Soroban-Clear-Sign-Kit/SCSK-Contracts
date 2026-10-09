#!/bin/bash
set -e

echo "Generating and funding alice..."
stellar keys generate --fund alice --network testnet || echo "Alice already exists"
echo "Generating and funding bob..."
stellar keys generate --fund bob --network testnet || echo "Bob already exists"
echo "Generating and funding issuer..."
stellar keys generate --fund issuer --network testnet || echo "Issuer already exists"

ALICE_PUB=$(stellar keys address alice)
BOB_PUB=$(stellar keys address bob)
ISSUER_PUB=$(stellar keys address issuer)
echo "Alice: $ALICE_PUB"
echo "Bob: $BOB_PUB"
echo "Issuer: $ISSUER_PUB"

echo "Building contracts..."
cargo build --target wasm32v1-none --release

echo "Deploying test asset SAC..."
SAC_ID=$(stellar contract asset deploy --asset "TST:$ISSUER_PUB" --source-account alice --network testnet)
echo "SAC ID: $SAC_ID"

echo "Deploying token contract (FIX)..."
TOKEN_ID=$(stellar contract deploy --wasm ../../target/wasm32v1-none/release/token.wasm --source-account alice --network testnet)
echo "Token ID: $TOKEN_ID"
stellar contract invoke --id $TOKEN_ID --source-account alice --network testnet -- initialize --admin $ALICE_PUB --decimals 7 --name "Fix Token" --symbol "FIX"

echo "Deploying look-alike USDC token contract..."
USDC_ID=$(stellar contract deploy --wasm ../../target/wasm32v1-none/release/token.wasm --source-account alice --network testnet)
echo "USDC ID: $USDC_ID"
stellar contract invoke --id $USDC_ID --source-account alice --network testnet -- initialize --admin $ALICE_PUB --decimals 18 --name "USDC Lookalike" --symbol "USDC"

echo "Deploying fixture contract..."
CONTRACT_ID=$(stellar contract deploy --wasm ../../target/wasm32v1-none/release/fixture.wasm --source-account alice --network testnet)
echo "Contract ID: $CONTRACT_ID"

echo "Minting balances..."
stellar contract invoke --id $TOKEN_ID --source-account alice --network testnet -- mint --to $ALICE_PUB --amount 10000000000
stellar contract invoke --id $USDC_ID --source-account alice --network testnet -- mint --to $ALICE_PUB --amount 10000000000

echo "Writing testnet.json..."
cat <<EOF > ../testnet.json
{
  "alice": "$ALICE_PUB",
  "bob": "$BOB_PUB",
  "issuer": "$ISSUER_PUB",
  "sacId": "$SAC_ID",
  "tokenId": "$TOKEN_ID",
  "usdcId": "$USDC_ID",
  "contractId": "$CONTRACT_ID"
}
EOF
echo "Done."
