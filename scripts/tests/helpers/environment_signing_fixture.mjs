import { randomBytes } from 'node:crypto';
import { ml_dsa65 } from '@noble/post-quantum/ml-dsa.js';
import { sha256 } from '@noble/hashes/sha2.js';

const fingerprint = (domain, key) => Buffer.from(sha256(Buffer.concat([
  Buffer.from(`${domain}\0`), Buffer.from(key),
]))).toString('hex');
const rootSeed = randomBytes(32);
const root = ml_dsa65.keygen(rootSeed);
const nodes = [];
try {
  for (const label of ['a', 'b']) {
    const seed = randomBytes(32);
    const key = ml_dsa65.keygen(seed);
    try {
      const nodeId = `test-metadata-${label}`;
      const publicKeyHex = Buffer.from(key.publicKey).toString('hex');
      const payload = { domain: 'rawrarr-device-approval-v1', nodeId, publicKeyHex, role: 'metadata' };
      const message = new TextEncoder().encode(JSON.stringify(payload, Object.keys(payload).sort()));
      const signature = ml_dsa65.sign(message, root.secretKey);
      if (!ml_dsa65.verify(signature, message, root.publicKey)) throw new Error('Invalid fixture signature');
      nodes.push({
        nodeId, seed: seed.toString('base64'), publicKeyHex, payload,
        fingerprint: fingerprint('rawrarr-device-signing-key-v1', key.publicKey),
        signature: Buffer.from(signature).toString('base64'),
      });
    } finally {
      seed.fill(0);
      key.secretKey.fill(0);
    }
  }
  // Captured only by the isolated fixture generator, never printed by its CLI.
  console.log(JSON.stringify({
    publicKey: Buffer.from(root.publicKey).toString('base64'),
    fingerprint: fingerprint('rawrarr-instance-root-v1', root.publicKey), nodes,
  }));
} finally {
  rootSeed.fill(0);
  root.secretKey.fill(0);
}
