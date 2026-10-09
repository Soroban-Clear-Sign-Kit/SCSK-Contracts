import { Keypair, TransactionBuilder, Networks, Contract, nativeToScVal, Account, Operation, FeeBumpTransaction, Asset, MuxedAccount } from '@stellar/stellar-sdk';
import fs from 'fs';
import path from 'path';

const NETWORK_PASSPHRASE = Networks.TESTNET;

async function main() {
  const testnetStr = fs.readFileSync(path.join(__dirname, '../testnet.json'), 'utf8');
  const { alice, bob, tokenId, usdcId } = JSON.parse(testnetStr);

  const envStr = fs.readFileSync(path.join(__dirname, '../.env'), 'utf8');
  const env = Object.fromEntries(envStr.split('\n').filter(Boolean).map(line => line.split('=')));
  const aliceSecret = env.ALICE_SECRET;
  
  const token = new Contract(tokenId);

  // We don't contact RPC, so we mock sequence numbers
  const source = new Account(alice, "100");
  
  const fixturesDir = path.join(__dirname, 'raw');
  fs.mkdirSync(fixturesDir, { recursive: true });

  const save = (name: string, tx: any) => {
    fs.writeFileSync(path.join(fixturesDir, `${name}.txt`), tx.toXDR());
  };

  // Base valid transaction
  const validTx = new TransactionBuilder(source, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(token.call('transfer', nativeToScVal(alice, { type: 'address' }), nativeToScVal(bob, { type: 'address' }), nativeToScVal(100, { type: 'i128' })))
    .setTimeout(0)
    .build();
  save('base_valid', validTx);

  // wrong_passphrase
  const wrongPassphraseTx = new TransactionBuilder(source, { fee: '10000', networkPassphrase: Networks.PUBLIC })
    .addOperation(token.call('transfer', nativeToScVal(alice, { type: 'address' }), nativeToScVal(bob, { type: 'address' }), nativeToScVal(100, { type: 'i128' })))
    .setTimeout(0)
    .build();
  save('wrong_passphrase', wrongPassphraseTx);

  // fee_bump_unwrap
  const feeBump = TransactionBuilder.buildFeeBumpTransaction(
    alice,
    '15000',
    validTx,
    NETWORK_PASSPHRASE
  );
  save('fee_bump_unwrap', feeBump);

  // classic_only
  const classicTx = new TransactionBuilder(source, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(Operation.payment({ destination: bob, asset: Asset.native(), amount: '10' }))
    .setTimeout(0)
    .build();
  save('classic_only', classicTx);

  // expired_time_bounds (minTime = 0, maxTime = 1) -> definitely expired
  const expiredTx = new TransactionBuilder(source, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE, timebounds: { minTime: 0, maxTime: 1 } })
    .addOperation(token.call('transfer', nativeToScVal(alice, { type: 'address' }), nativeToScVal(bob, { type: 'address' }), nativeToScVal(100, { type: 'i128' })))
    .build();
  save('expired_time_bounds', expiredTx);

  // muxed_source
  const muxedAccount = new MuxedAccount(source, "1234");
  const muxedTx = new TransactionBuilder(muxedAccount, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(token.call('transfer', nativeToScVal(alice, { type: 'address' }), nativeToScVal(bob, { type: 'address' }), nativeToScVal(100, { type: 'i128' })))
    .setTimeout(0)
    .build();
  save('muxed_source', muxedTx);

  console.log('Capture done');
}

main().catch(console.error);
