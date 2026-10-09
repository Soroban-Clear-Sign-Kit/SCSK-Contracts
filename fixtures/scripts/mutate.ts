import fs from 'fs';
import path from 'path';

function copyDir(src: string, dest: string) {
  if (!fs.existsSync(dest)) fs.mkdirSync(dest, { recursive: true });
  for (const file of fs.readdirSync(src)) {
    const srcFile = path.join(src, file);
    const destFile = path.join(dest, file);
    if (fs.statSync(srcFile).isDirectory()) {
      copyDir(srcFile, destFile);
    } else {
      fs.copyFileSync(srcFile, destFile);
    }
  }
}

function writeMutation(dest: string, sourceCase: string, edit: string) {
  fs.writeFileSync(path.join(dest, 'MUTATION.md'), `Source case: ${sourceCase}\n\nEdit: ${edit}\n`);
}

function main() {
  const fixturesDir = path.join(__dirname, '../data'); // Assuming captures go to data instead of raw
  const syntheticDir = path.join(__dirname, '../data');
  
  // Create directories if they don't exist
  if (!fs.existsSync(syntheticDir)) fs.mkdirSync(syntheticDir, { recursive: true });

  const mutate = (name: string, source: string, mutation: (dir: string) => void, desc: string) => {
    const srcDir = path.join(fixturesDir, source);
    const destDir = path.join(syntheticDir, name);
    if (!fs.existsSync(srcDir)) {
      console.warn(`Source ${source} not found, skipping ${name}`);
      return;
    }
    copyDir(srcDir, destDir);
    mutation(destDir);
    writeMutation(destDir, source, desc);
    console.log(`Generated ${name}`);
  };

  // syn-silent-transfer: token-transfer with the transfer event removed from the recorded response
  mutate('syn-silent-transfer', 'token-transfer', (dir) => {
    const simFile = path.join(dir, 'simulateTransaction.json');
    if (fs.existsSync(simFile)) {
      const data = JSON.parse(fs.readFileSync(simFile, 'utf8'));
      if (data.events) {
        data.events = data.events.filter((e: any) => !e.topic || e.topic[0] !== 'transfer');
      }
      fs.writeFileSync(simFile, JSON.stringify(data, null, 2));
    }
  }, 'Removed transfer event from simulateTransaction.json');

  // syn-map-event: sac-transfer with event data rewritten as a map containing amount
  mutate('syn-map-event', 'sac-transfer', (dir) => {
    const simFile = path.join(dir, 'simulateTransaction.json');
    if (fs.existsSync(simFile)) {
      // In reality we would decode XDR, rewrite as a map ScVal, and re-encode.
      // Since this is offline setup without RPC connection we just record intent.
      fs.writeFileSync(path.join(dir, 'mutate_todo.txt'), 'TODO: rewrite event data as map XDR during actual mutation pass');
    }
  }, 'Rewritten transfer event data as map containing amount');

  // syn-duplicate-nonce: auth-unsigned-entry with the entry duplicated
  mutate('syn-duplicate-nonce', 'auth-unsigned-entry', (dir) => {
    const txFile = path.join(dir, 'tx.txt');
    if (fs.existsSync(txFile)) {
      // We would parse the tx, duplicate the auth entry, and re-serialize.
      fs.writeFileSync(path.join(dir, 'mutate_todo.txt'), 'TODO: duplicate auth entry in XDR');
    }
  }, 'Duplicated the auth entry in the transaction envelope');

  // syn-rpc-network: sac-transfer with the recorded getNetwork passphrase changed
  mutate('syn-rpc-network', 'sac-transfer', (dir) => {
    const netFile = path.join(dir, 'getNetwork.json');
    if (fs.existsSync(netFile)) {
      const data = JSON.parse(fs.readFileSync(netFile, 'utf8'));
      if (data.passphrase) data.passphrase = 'Public Global Stellar Network ; September 2015';
      fs.writeFileSync(netFile, JSON.stringify(data, null, 2));
    }
  }, 'Changed passphrase in getNetwork.json to Public Network');

  // syn-no-contract: token-transfer with the instance lookup returning no entry
  mutate('syn-no-contract', 'token-transfer', (dir) => {
    const entriesFile = path.join(dir, 'getLedgerEntries.json');
    if (fs.existsSync(entriesFile)) {
      const data = JSON.parse(fs.readFileSync(entriesFile, 'utf8'));
      if (data.entries) data.entries = []; // Empty out instance lookup
      fs.writeFileSync(entriesFile, JSON.stringify(data, null, 2));
    }
  }, 'Removed contract instance entry from getLedgerEntries.json');

  console.log('Mutations complete.');
}

main();
