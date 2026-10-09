$ErrorActionPreference = "Stop"

Write-Host "Generating and funding alice..."
try { stellar keys generate --fund alice --network testnet } catch { Write-Host "Alice already exists" }

Write-Host "Generating and funding bob..."
try { stellar keys generate --fund bob --network testnet } catch { Write-Host "Bob already exists" }

$ALICE_PUB = stellar keys address alice
$BOB_PUB = stellar keys address bob
Write-Host "Alice: $ALICE_PUB"
Write-Host "Bob: $BOB_PUB"

Write-Host "Deploying test asset SAC..."
$RAND = Get-Random -Maximum 100000
$ASSET_CODE = "TST$RAND"
$TOKEN_ID = stellar contract asset deploy --asset "${ASSET_CODE}:$ALICE_PUB" --source-account alice --network testnet
Write-Host "Token ID: $TOKEN_ID"


Write-Host "Building fixture contract..."
cargo build --target wasm32v1-none --release

Write-Host "Deploying fixture contract..."
$CONTRACT_ID = stellar contract deploy --wasm ../../target/wasm32v1-none/release/fixture.wasm --source-account alice --network testnet
Write-Host "Contract ID: $CONTRACT_ID"

Write-Host "Writing testnet.json and .env..."
$ALICE_SEC = stellar keys show alice
$BOB_SEC = stellar keys show bob
$jsonObj = @{
    alice = $ALICE_PUB
    bob = $BOB_PUB
    tokenId = $TOKEN_ID
    contractId = $CONTRACT_ID
    assetCode = $ASSET_CODE
}
$jsonObj | ConvertTo-Json | Out-File -Encoding ASCII ../testnet.json

$envContent = "ALICE_SECRET=$ALICE_SEC`nBOB_SECRET=$BOB_SEC`n"
Set-Content -Path ../.env -Value $envContent -Encoding ASCII

Write-Host "Creating trustline for Bob and minting..."
node trustline.js
