import { Keypair, rpc, TransactionBuilder, xdr, Networks, Contract, Address, nativeToScVal, Account, Operation, FeeBumpTransaction, Asset } from '@stellar/stellar-sdk';
import fs from 'fs';
import path from 'path';

const RPC_URL = 'https://soroban-testnet.stellar.org';
const NETWORK_PASSPHRASE = Networks.TESTNET;
const server = new rpc.Server(RPC_URL);

async function getSourceAccount(publicKey: string) {
  const acc = await server.getAccount(publicKey);
  return new Account(acc.accountId(), acc.sequenceNumber());
}

async function simulateAndSave(name: string, tx: any, modifyAuth?: (simResult: any) => void) {
  const sim = await server.simulateTransaction(tx);
  if (modifyAuth && rpc.Api.isSimulationSuccess(sim) && sim.result.auth) {
     modifyAuth(sim);
     // Re-encode with modified auth, wait actually simulation returns it. We can just modify the JSON before saving.
  }
  const fixturesDir = path.join(__dirname, '../../../SCSK-Backend/test/fixtures');
  fs.mkdirSync(fixturesDir, { recursive: true });
  fs.writeFileSync(path.join(fixturesDir, `${name}.json`), JSON.stringify({
     xdr: tx.toXDR(),
     sim
  }, null, 2));
}


async function main() {
  const testnetStr = fs.readFileSync(path.join(__dirname, '../testnet.json'), 'utf8');
  const { alice, bob, tokenId, contractId } = JSON.parse(testnetStr);

  const envStr = fs.readFileSync(path.join(__dirname, '../.env'), 'utf8');
  const env = Object.fromEntries(envStr.split('\n').filter(Boolean).map(line => line.split('=')));
  const aliceSecret = env.ALICE_SECRET;
  const bobSecret = env.BOB_SECRET;

  const contract = new Contract(contractId);
  const token = new Contract(tokenId);

  const aliceKp = Keypair.fromSecret(aliceSecret);
  const bobKp = Keypair.fromSecret(bobSecret);

  // Case 1: SAC transfer
  const source1 = await getSourceAccount(alice);
  const tx1 = new TransactionBuilder(source1, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(token.call('transfer', nativeToScVal(alice, { type: 'address' }), nativeToScVal(bob, { type: 'address' }), nativeToScVal(100, { type: 'i128' })))
    .setTimeout(0)
    .build();
  await simulateAndSave('sac_transfer', tx1);

  // Case 2: fixture forward (nested auth: forwarder to token)
  const source2 = await getSourceAccount(alice);
  const tx2 = new TransactionBuilder(source2, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(contract.call('forward', 
       nativeToScVal(alice, { type: 'address' }), 
       nativeToScVal(tokenId, { type: 'address' }), 
       nativeToScVal(bob, { type: 'address' }), 
       nativeToScVal(100, { type: 'i128' })))
    .setTimeout(0)
    .build();
  await simulateAndSave('fixture_forward', tx2);

  // Case 3: forward_with_meta
  const source3 = await getSourceAccount(alice);
  const tx3 = new TransactionBuilder(source3, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(contract.call('forward_with_meta', 
       nativeToScVal(alice, { type: 'address' }), 
       nativeToScVal(tokenId, { type: 'address' }), 
       nativeToScVal(bob, { type: 'address' }), 
       nativeToScVal(100, { type: 'i128' }),
       nativeToScVal({ memo: 'hello', tags: [] }),
       nativeToScVal({ Fast: undefined }) // enum Fast
    ))
    .setTimeout(0)
    .build();
  await simulateAndSave('forward_with_meta', tx3);

  // Case 4: wrong network passphrase
  const tx4 = new TransactionBuilder(source3, { fee: '10000', networkPassphrase: Networks.PUBLIC })
    .addOperation(token.call('transfer', nativeToScVal(alice, { type: 'address' }), nativeToScVal(bob, { type: 'address' }), nativeToScVal(100, { type: 'i128' })))
    .setTimeout(0)
    .build();
  await simulateAndSave('wrong_network', tx4);

  // Case 5: expired auth entry
  const source5 = await getSourceAccount(bob);
  let tx5 = new TransactionBuilder(source5, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(token.call('transfer', nativeToScVal(alice, { type: 'address' }), nativeToScVal(bob, { type: 'address' }), nativeToScVal(100, { type: 'i128' })))
    .setTimeout(0)
    .build();
  
  const address = xdr.ScAddress.scAddressTypeAccount(
      xdr.PublicKey.publicKeyTypeEd25519(aliceKp.rawPublicKey())
  );
  
  const credentials = xdr.SorobanCredentials.sorobanCredentialsAddress(
      new xdr.SorobanAddressCredentials({
          address: address,
          nonce: xdr.Int64(0),
          signatureExpirationLedger: 10, // Expired ledger!
          signature: xdr.ScVal.scvVoid()
      })
  );
  
  const auth = new xdr.SorobanAuthorizationEntry({
      credentials: credentials,
      rootInvocation: new xdr.SorobanAuthorizedInvocation({
          function: xdr.SorobanAuthorizedFunction.sorobanAuthorizedFunctionTypeContractFn(
              new xdr.InvokeContractArgs({
                  contractAddress: new Contract(tokenId).address().toScAddress(),
                  functionName: 'transfer',
                  args: [
                      nativeToScVal(alice, { type: 'address' }),
                      nativeToScVal(bob, { type: 'address' }),
                      nativeToScVal(100, { type: 'i128' })
                  ]
              })
          ),
          subInvocations: []
      })
  });
  
  tx5 = new TransactionBuilder(new Account(tx5.source, (BigInt(tx5.sequence) - 1n).toString()), { fee: tx5.fee, networkPassphrase: NETWORK_PASSPHRASE })
      .addOperation(Operation.invokeHostFunction({
          func: (tx5.operations[0] as any).func,
          auth: [auth]
      }))
      .setTimeout(0)
      .build();
  // No need to pass modifyAuth callback, we modified the XDR directly
  await simulateAndSave('expired_auth', tx5);

  // Case 6: simulation failure (amount above balance)
  const source6 = await getSourceAccount(alice);
  const tx6 = new TransactionBuilder(source6, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(token.call('transfer', nativeToScVal(alice, { type: 'address' }), nativeToScVal(bob, { type: 'address' }), nativeToScVal('1000000000000000000000000', { type: 'i128' })))
    .setTimeout(0)
    .build();
  await simulateAndSave('sim_fail_balance', tx6);

  // Case 7: a fee-bump wrapping a transfer
  const feeBump = TransactionBuilder.buildFeeBumpTransaction(
    alice,
    '15000',
    tx1,
    NETWORK_PASSPHRASE
  );
  const sim7 = await server.simulateTransaction(tx1); // fee bumps use inner tx for sim
  const fixturesDir = path.join(__dirname, '../../../SCSK-Backend/test/fixtures');
  fs.writeFileSync(path.join(fixturesDir, `feebump.json`), JSON.stringify({
     xdr: feeBump.toXDR(),
     sim: sim7
  }, null, 2));

  // Case 8: a classic payment
  const tx8 = new TransactionBuilder(source1, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(Operation.payment({ destination: bob, asset: Asset.native(), amount: '10' }))
    .setTimeout(0)
    .build();
  // Classic txs don't simulate on Soroban RPC
  fs.writeFileSync(path.join(fixturesDir, `classic.json`), JSON.stringify({
     xdr: tx8.toXDR(),
     sim: null
  }, null, 2));

  console.log('Capture done');
}

main().catch(console.error);
