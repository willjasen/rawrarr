import assert from 'node:assert/strict';
import test from 'node:test';
import { Decrypter, armor, generateHybridIdentity, identityToRecipient } from '../../rust-arr/web-export/node_modules/age-encryption/dist/index.js';
import { createEncryptedRootBackup, inspectEncryptedRootBackup, publicKeyFingerprint, signDeviceApproval, verifyDeviceApproval } from '../../rust-arr/web-export/device-signing-client.js';

function decodeBase64(value) {
  return Uint8Array.from(atob(value), character => character.charCodeAt(0));
}

test('age-encrypted ML-DSA root backup signs and verifies a device approval', async () => {
  const ageIdentity = await generateHybridIdentity();
  const ageRecipient = await identityToRecipient(ageIdentity);
  const root = await createEncryptedRootBackup(ageRecipient);
  assert.equal(root.algorithm, 'ML-DSA-65');
  assert.match(root.fingerprint, /^[0-9a-f]{64}$/);
  assert.match(root.encryptedBackup, /^-----BEGIN AGE ENCRYPTED FILE-----/);
  assert.deepEqual(await inspectEncryptedRootBackup(root.encryptedBackup, ageIdentity), {
    algorithm: root.algorithm, publicKey: root.publicKey, fingerprint: root.fingerprint,
  });
  const foldedBackup = root.encryptedBackup.replace(/\r?\n/g, ' ');
  assert.deepEqual(await inspectEncryptedRootBackup(foldedBackup, ageIdentity), {
    algorithm: root.algorithm, publicKey: root.publicKey, fingerprint: root.fingerprint,
  });

  const request = {
    instanceId: 'instance-fixture',
    requestId: 'request-fixture',
    nodeId: 'device-fixture',
    role: 'storage',
    displayName: 'Storage fixture',
    publicKey: 'dGVzdA==',
  };
  const approval = await signDeviceApproval(foldedBackup, ageIdentity, root.fingerprint, request);
  assert.equal(approval.rootFingerprint, root.fingerprint);
  assert.equal(verifyDeviceApproval(approval.signature, root.publicKey, approval.signedPayload), true);
  assert.equal(verifyDeviceApproval(approval.signature, root.publicKey, { ...approval.signedPayload, role: 'metadata' }), false);
  await assert.rejects(
    signDeviceApproval(root.encryptedBackup, ageIdentity, '0'.repeat(64), request),
    /does not match the instance signing root/,
  );

  const wrongIdentity = await generateHybridIdentity();
  await assert.rejects(inspectEncryptedRootBackup(root.encryptedBackup, wrongIdentity), /Could not unlock/);
  await assert.rejects(
    signDeviceApproval(root.encryptedBackup, wrongIdentity, root.fingerprint, request),
    /Could not unlock or validate/,
  );
});

test('root fingerprint is domain-separated and stable', async () => {
  const ageIdentity = await generateHybridIdentity();
  const root = await createEncryptedRootBackup(await identityToRecipient(ageIdentity));
  const plaintext = await (async () => {
    const decrypter = new Decrypter();
    decrypter.addIdentity(ageIdentity);
    return decrypter.decrypt(armor.decode(root.encryptedBackup));
  })();
  const record = JSON.parse(new TextDecoder().decode(plaintext));
  const publicKey = decodeBase64(record.publicKey);
  assert.equal(await publicKeyFingerprint(publicKey), root.fingerprint);
  plaintext.fill(0);
  publicKey.fill(0);
});
