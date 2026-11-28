import assert from 'node:assert/strict';
import test from 'node:test';
import { Decrypter, Encrypter, armor, generateHybridIdentity, identityToRecipient } from '../../rust-arr/web-export/node_modules/age-encryption/dist/index.js';
import { createAppKeyGrant, createEncryptedRootBackup, decryptOwnerAuthorizedAppIdentity, inspectEncryptedRootBackup, publicKeyFingerprint, signDeviceApproval, signDeviceRejection, verifyDeviceApproval } from '../../rust-arr/web-export/device-signing-client.js';
import { createAppKeyGrant as createBundledAppKeyGrant } from '../../rust-arr/src/assets/device-signing-client.js';

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
  const rejection = await signDeviceRejection(foldedBackup, ageIdentity, root.fingerprint, request);
  assert.equal(rejection.signedPayload.domain, 'rawrarr-device-rejection-v1');
  assert.equal(verifyDeviceApproval(rejection.signature, root.publicKey, rejection.signedPayload), true);
  assert.equal(verifyDeviceApproval(approval.signature, root.publicKey, rejection.signedPayload), false);
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

test('owner authorizes an app identity locally and creates a fossil-bound encrypted grant', async () => {
  const ownerIdentity = await generateHybridIdentity();
  const ownerRecipient = await identityToRecipient(ownerIdentity);
  const groupIdentity = await generateHybridIdentity();
  const groupRecipient = await identityToRecipient(groupIdentity);
  const appIdentity = await generateHybridIdentity();
  const appRecipient = await identityToRecipient(appIdentity);
  const deviceIdentity = await generateHybridIdentity();
  const deviceRecipient = await identityToRecipient(deviceIdentity);
  const appInstanceId = '723e4567-e89b-42d3-a456-426614174000';
  const grantOwnerFile = async (recipient, bytes) => {
    const encryptor = new Encrypter();
    encryptor.addRecipient(recipient);
    return new Uint8Array(await encryptor.encrypt(bytes));
  };
  const groupEnvelope = await grantOwnerFile(ownerRecipient, new TextEncoder().encode(groupIdentity));
  const registry = new TextEncoder().encode(JSON.stringify({
    schemaVersion: 1,
    apps: { radarr: { instanceId: appInstanceId, identity: appIdentity, recipient: appRecipient } },
  }));
  const registryCiphertext = await grantOwnerFile(groupRecipient, registry);
  const authorized = await decryptOwnerAuthorizedAppIdentity(
    ownerIdentity, groupEnvelope, registryCiphertext, 'radarr', appInstanceId,
  );
  assert.equal(authorized.appIdentity, appIdentity);
  assert.equal(authorized.appRecipient, appRecipient);

  const root = await createEncryptedRootBackup(ownerRecipient);
  const grant = await createAppKeyGrant(root.encryptedBackup, ownerIdentity, root.fingerprint, {
    ...authorized,
    deviceId: 'fossil-device-17',
    deviceRecipient,
    keyEpoch: 1,
    minimumKeyEpoch: 0,
  });
  assert.equal(grant.algorithm, 'ML-DSA-65');
  assert.equal(grant.rootFingerprint, root.fingerprint);
  assert.equal(grant.rootPublicKey, root.publicKey);
  assert.deepEqual(grant.signedPayload, {
    domain: 'rawrarr-app-key-grant-v1',
    schemaVersion: 1,
    appInstanceId,
    appType: 'radarr',
    appRecipient,
    deviceId: 'fossil-device-17',
    deviceRecipient,
    keyEpoch: 1,
    encryptedIdentitySha256: grant.signedPayload.encryptedIdentitySha256,
  });
  assert.equal(verifyDeviceApproval(grant.signature, grant.rootPublicKey, grant.signedPayload), true);

  const bundledGrant = await createBundledAppKeyGrant(root.encryptedBackup, ownerIdentity, root.fingerprint, {
    ...authorized,
    deviceId: 'fossil-device-17',
    deviceRecipient,
    keyEpoch: 1,
    minimumKeyEpoch: 0,
  });
  assert.equal(verifyDeviceApproval(bundledGrant.signature, bundledGrant.rootPublicKey, bundledGrant.signedPayload), true);
  assert.equal(bundledGrant.signedPayload.encryptedIdentitySha256.length, 64);

  const envelope = decodeBase64(grant.encryptedIdentity);
  assert.equal(new TextDecoder().decode(envelope).includes(appIdentity), false);
  const decryptor = new Decrypter();
  decryptor.addIdentity(deviceIdentity);
  const opened = await decryptor.decrypt(envelope);
  assert.equal(new TextDecoder().decode(opened), appIdentity.trim());
  const wrongDevice = new Decrypter();
  wrongDevice.addIdentity(ownerIdentity);
  await assert.rejects(wrongDevice.decrypt(envelope));

  for (const field of ['appInstanceId', 'appType', 'appRecipient', 'deviceId', 'deviceRecipient', 'keyEpoch', 'encryptedIdentitySha256']) {
    const value = grant.signedPayload[field];
    const altered = field === 'keyEpoch' ? value + 1 : `${value}x`;
    assert.equal(verifyDeviceApproval(grant.signature, grant.rootPublicKey, { ...grant.signedPayload, [field]: altered }), false, `${field} is signed`);
  }
  const changedEnvelope = await grantOwnerFile(deviceRecipient, new TextEncoder().encode('different identity'));
  const changedDigest = [...new Uint8Array(await crypto.subtle.digest('SHA-256', changedEnvelope))]
    .map(byte => byte.toString(16).padStart(2, '0')).join('');
  assert.notEqual(changedDigest, grant.signedPayload.encryptedIdentitySha256);
  const staleEpoch = { ...authorized, deviceId: 'fossil-device-17', deviceRecipient, keyEpoch: 1, minimumKeyEpoch: 1 };
  await assert.rejects(
    createAppKeyGrant(root.encryptedBackup, ownerIdentity, root.fingerprint, staleEpoch),
    /greater than the target fossil’s accepted epoch/,
  );
  await assert.rejects(
    createAppKeyGrant(root.encryptedBackup, ownerIdentity, '0'.repeat(64), {
      ...authorized, deviceId: 'fossil-device-17', deviceRecipient, keyEpoch: 1, minimumKeyEpoch: 0,
    }),
    /does not match the instance signing root/,
  );
  await assert.rejects(
    decryptOwnerAuthorizedAppIdentity(ownerIdentity, groupEnvelope, registryCiphertext, 'sonarr', appInstanceId),
    /unavailable for the selected instance/,
  );
  envelope.fill(0);
  opened.fill(0);
  groupEnvelope.fill(0);
  registry.fill(0);
  registryCiphertext.fill(0);
});

test('owner app-key authorization rejects malformed or oversized registry ciphertext', async () => {
  const ownerIdentity = await generateHybridIdentity();
  await assert.rejects(
    decryptOwnerAuthorizedAppIdentity(ownerIdentity, new Uint8Array(), new Uint8Array(), 'radarr', '723e4567-e89b-42d3-a456-426614174000'),
    /missing or oversized/,
  );
  const ownerRecipient = await identityToRecipient(ownerIdentity);
  const groupIdentity = await generateHybridIdentity();
  const groupRecipient = await identityToRecipient(groupIdentity);
  const encryptTo = async (recipient, bytes) => {
    const encryptor = new Encrypter();
    encryptor.addRecipient(recipient);
    return new Uint8Array(await encryptor.encrypt(bytes));
  };
  const groupEnvelope = await encryptTo(ownerRecipient, new TextEncoder().encode(groupIdentity));
  const invalidRegistry = await encryptTo(groupRecipient, new TextEncoder().encode('{"schemaVersion":2,"apps":{}}'));
  await assert.rejects(
    decryptOwnerAuthorizedAppIdentity(ownerIdentity, groupEnvelope, invalidRegistry, 'radarr', '723e4567-e89b-42d3-a456-426614174000'),
    /unsupported format/,
  );
  groupEnvelope.fill(0);
  invalidRegistry.fill(0);
});
