import { Keypair, rpc, TransactionBuilder, xdr, Networks, Contract, Address, nativeToScVal, Account, Operation } from '@stellar/stellar-sdk';
import fs from 'fs';
import path from 'path';
import crypto from 'crypto';

const RPC_URL = 'https://soroban-testnet.stellar.org';
const NETWORK_PASSPHRASE = Networks.TESTNET;
const server = new rpc.Server(RPC_URL);

async function fundAccount(publicKey: string) {
  const res = await fetch(`https://friendbot.stellar.org/?addr=${encodeURIComponent(publicKey)}`);
  if (!res.ok) throw new Error(`Friendbot failed: ${await res.text()}`);
}

async function getSourceAccount(publicKey: string) {
  const acc = await server.getAccount(publicKey);
  return new Account(acc.accountId(), acc.sequenceNumber());
}

async function uploadWasm(admin: Keypair, wasmPath: string): Promise<string> {
  const wasm = fs.readFileSync(wasmPath);
  const source = await getSourceAccount(admin.publicKey());
  const tx = new TransactionBuilder(source, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(Operation.uploadContractWasm({ wasm }))
    .setTimeout(300)
    .build();

  const sim = await server.simulateTransaction(tx);

  if (!rpc.Api.isSimulationSuccess(sim)) throw new Error('Upload failed');
  
  const prepared = await server.prepareTransaction(tx);
  prepared.sign(admin);
  const resp = await server.sendTransaction(prepared);
  if (resp.status === 'ERROR') throw new Error('Send failed');
  
  // wait for completion
  let status = await server.getTransaction(resp.hash);
  while (status.status === 'NOT_FOUND') {
    await new Promise(r => setTimeout(r, 2000));
    status = await server.getTransaction(resp.hash);
  }
  
  if (status.status !== 'SUCCESS') throw new Error('Tx failed');
  
  const wasmHash = crypto.createHash('sha256').update(wasm).digest('hex');
  return wasmHash;
}

async function createContract(admin: Keypair, wasmHashHex: string): Promise<string> {
  const source = await getSourceAccount(admin.publicKey());
  const salt = Buffer.alloc(32);
  const op = Operation.createCustomContract({
    address: Address.fromString(admin.publicKey()),
    wasmHash: Buffer.from(wasmHashHex, 'hex'),
    salt
  });
  
  const tx = new TransactionBuilder(source, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(op)
    .setTimeout(300)
    .build();
    
  const sim = await server.simulateTransaction(tx);
  if (!rpc.Api.isSimulationSuccess(sim)) throw new Error('Create failed');
  
  const prepared = await server.prepareTransaction(tx);
  prepared.sign(admin);
  const resp = await server.sendTransaction(prepared);
  
  let status = await server.getTransaction(resp.hash);
  while (status.status === 'NOT_FOUND') {
    await new Promise(r => setTimeout(r, 2000));
    status = await server.getTransaction(resp.hash);
  }
  
  if (status.status !== 'SUCCESS') throw new Error('Create tx failed');

  const networkId = crypto.createHash('sha256').update(NETWORK_PASSPHRASE).digest();
  const preimage = xdr.HashIdPreimage.envelopeTypeContractId(
    new xdr.HashIdPreimageContractId({
      networkId,
      contractIdPreimage: xdr.ContractIdPreimage.contractIdPreimageFromAddress(
        new xdr.ContractIdPreimageFromAddress({
          address: Address.fromString(admin.publicKey()).toScAddress(),
          salt
        })
      )
    })
  );
  const contractId = crypto.createHash('sha256').update(preimage.toXDR()).digest('hex');
  return Address.contract(Buffer.from(contractId, 'hex')).toString();
}

async function main() {
  const admin = Keypair.random();
  console.log('Admin:', admin.publicKey());
  await fundAccount(admin.publicKey());
  
  console.log('Uploading wasm...');
  const wasmPath = path.join(__dirname, '../../target/wasm32v1-none/release/fixture.wasm');
  const wasmHash = await uploadWasm(admin, wasmPath);
  
  console.log('Creating contract...');
  const contractId = await createContract(admin, wasmHash);
  console.log('Contract ID:', contractId);
  
  const contract = new Contract(contractId);
  
  const fixturesDir = path.join(__dirname, '../../../../SCSK-Backend/test/fixtures');
  fs.mkdirSync(fixturesDir, { recursive: true });
  
  const source1 = await getSourceAccount(admin.publicKey());
  const tx1 = new TransactionBuilder(source1, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(contract.call('simple_call', nativeToScVal(123, { type: 'i32' }), nativeToScVal('hello'), nativeToScVal(admin.publicKey(), { type: 'address' })))
    .setTimeout(300)
    .build();
    
  const sim1 = await server.simulateTransaction(tx1);
  fs.writeFileSync(path.join(fixturesDir, 'simple_call.json'), JSON.stringify({
     xdr: tx1.toXDR(),
     sim: sim1
  }, null, 2));

  const source2 = await getSourceAccount(admin.publicKey());
  const toUser = Keypair.random();
  const tx2 = new TransactionBuilder(source2, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(contract.call('transfer_auth', 
       nativeToScVal(admin.publicKey(), { type: 'address' }), 
       nativeToScVal(toUser.publicKey(), { type: 'address' }), 
       nativeToScVal(100, { type: 'i128' })))
    .setTimeout(300)
    .build();
    
  const sim2 = await server.simulateTransaction(tx2);
  fs.writeFileSync(path.join(fixturesDir, 'transfer_auth.json'), JSON.stringify({
     xdr: tx2.toXDR(),
     sim: sim2
  }, null, 2));
  
  const source3 = await getSourceAccount(admin.publicKey());
  const tx3 = new TransactionBuilder(source3, { fee: '10000', networkPassphrase: NETWORK_PASSPHRASE })
    .addOperation(contract.call('nested_auth', 
       nativeToScVal(admin.publicKey(), { type: 'address' }), 
       nativeToScVal(toUser.publicKey(), { type: 'address' }), 
       nativeToScVal(50, { type: 'i128' })))
    .setTimeout(300)
    .build();
    
  const sim3 = await server.simulateTransaction(tx3);
  fs.writeFileSync(path.join(fixturesDir, 'nested_auth.json'), JSON.stringify({
     xdr: tx3.toXDR(),
     sim: sim3
  }, null, 2));
  
  console.log('Done generating fixtures.');

  
  fs.writeFileSync(path.join(fixturesDir, 'testnet.json'), JSON.stringify({
     rpcUrl: RPC_URL,
     networkPassphrase: NETWORK_PASSPHRASE,
     adminPublic: admin.publicKey(),
     contractId
  }, null, 2));

  fs.writeFileSync(path.join(fixturesDir, '.env'), `ADMIN_SECRET=${admin.secret()}\n`);
}

main().catch(console.error);
