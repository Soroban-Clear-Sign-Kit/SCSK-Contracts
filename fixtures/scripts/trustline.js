const { Keypair, Asset, TransactionBuilder, Operation, Account } = require('@stellar/stellar-sdk');
const rpc = require('@stellar/stellar-sdk').rpc;
const fs = require('fs');

async function main() {
    try {
        const s = new rpc.Server('https://soroban-testnet.stellar.org');
        const testnetData = JSON.parse(fs.readFileSync('../testnet.json', 'utf8'));
        const { aliceSecret, bobSecret, assetCode } = testnetData;

        const aliceKey = Keypair.fromSecret(aliceSecret);
        const bobKey = Keypair.fromSecret(bobSecret);
        const asset = new Asset(assetCode, aliceKey.publicKey());

        const bobAccount = await s.getAccount(bobKey.publicKey());
        let tx = new TransactionBuilder(bobAccount, {fee: '10000', networkPassphrase: 'Test SDF Network ; September 2015'})
            .addOperation(Operation.changeTrust({ asset: asset }))
            .setTimeout(300)
            .build();
        tx.sign(bobKey);
        await s.sendTransaction(tx);
        console.log("Bob trusted", assetCode);

        const aliceAccount = await s.getAccount(aliceKey.publicKey());
        let tx2 = new TransactionBuilder(aliceAccount, {fee: '10000', networkPassphrase: 'Test SDF Network ; September 2015'})
            .addOperation(Operation.payment({ destination: bobKey.publicKey(), asset: asset, amount: '100' }))
            .setTimeout(300)
            .build();
        tx2.sign(aliceKey);
        await s.sendTransaction(tx2);
        console.log("Alice minted to Bob", assetCode);
    } catch(e) {
        console.error(e);
    }
}
main();
