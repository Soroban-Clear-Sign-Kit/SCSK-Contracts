import fs from 'fs';
import path from 'path';

function main() {
  const rawDir = path.join(__dirname, 'raw');
  const outDir = path.join(__dirname, '../../../SCSK-Backend/test/fixtures');
  
  if (!fs.existsSync(outDir)) {
    fs.mkdirSync(outDir, { recursive: true });
  }

  const baseValid = fs.readFileSync(path.join(rawDir, 'base_valid.txt'), 'utf8');

  // empty
  fs.writeFileSync(path.join(outDir, 'empty.json'), JSON.stringify({ xdr: "" }, null, 2));

  // oversized (MAX_XDR_BASE64_LENGTH = 200000)
  fs.writeFileSync(path.join(outDir, 'oversized.json'), JSON.stringify({ xdr: baseValid + "A".repeat(200000) }, null, 2));

  // bad_base64
  fs.writeFileSync(path.join(outDir, 'bad_base64.json'), JSON.stringify({ xdr: baseValid + "!!!INVALID@@@" }, null, 2));

  // truncated
  fs.writeFileSync(path.join(outDir, 'truncated.json'), JSON.stringify({ xdr: baseValid.substring(0, 10) }, null, 2));

  // wrong_passphrase
  const wrongPassphrase = fs.readFileSync(path.join(rawDir, 'wrong_passphrase.txt'), 'utf8');
  fs.writeFileSync(path.join(outDir, 'wrong_passphrase.json'), JSON.stringify({ xdr: wrongPassphrase }, null, 2));

  // fee_bump_unwrap
  const feeBump = fs.readFileSync(path.join(rawDir, 'fee_bump_unwrap.txt'), 'utf8');
  fs.writeFileSync(path.join(outDir, 'fee_bump_unwrap.json'), JSON.stringify({ xdr: feeBump }, null, 2));

  // classic_only
  const classicOnly = fs.readFileSync(path.join(rawDir, 'classic_only.txt'), 'utf8');
  fs.writeFileSync(path.join(outDir, 'classic_only.json'), JSON.stringify({ xdr: classicOnly }, null, 2));

  // expired_time_bounds
  const expired = fs.readFileSync(path.join(rawDir, 'expired_time_bounds.txt'), 'utf8');
  fs.writeFileSync(path.join(outDir, 'expired_time_bounds.json'), JSON.stringify({ xdr: expired }, null, 2));

  // muxed_source
  const muxed = fs.readFileSync(path.join(rawDir, 'muxed_source.txt'), 'utf8');
  fs.writeFileSync(path.join(outDir, 'muxed_source.json'), JSON.stringify({ xdr: muxed }, null, 2));

  console.log('Mutations complete.');
}

main();
