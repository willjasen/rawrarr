import { Decrypter, Encrypter, armor, identityToRecipient } from 'age-encryption';
import { ml_dsa65 } from '@noble/post-quantum/ml-dsa.js';
import { sha256 } from '@noble/hashes/sha2.js';

const encoder = new TextEncoder();
const decoder = new TextDecoder('utf-8', { fatal: true });
const ROOT_BACKUP_VERSION = 1;
const ROOT_DOMAIN = encoder.encode('rawrarr-instance-root-v1\0');
const APPROVAL_DOMAIN = 'rawrarr-device-approval-v1';
const REVOCATION_DOMAIN = 'rawrarr-device-rejection-v1';
const APP_KEY_GRANT_DOMAIN = 'rawrarr-app-key-grant-v1';
const MAX_APP_REGISTRY_BYTES = 64 * 1024;
const MAX_APP_IDENTITY_BYTES = 16 * 1024;

function toBase64(bytes) {
  let binary = '';
  for (let offset = 0; offset < bytes.length; offset += 0x8000) {
    binary += String.fromCharCode(...bytes.subarray(offset, offset + 0x8000));
  }
  return btoa(binary);
}

function fromBase64(value, expectedLength, label) {
  if (typeof value !== 'string' || !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(value)) {
    throw new Error(`Invalid ${label} encoding.`);
  }
  const binary = atob(value);
  const bytes = Uint8Array.from(binary, character => character.charCodeAt(0));
  if (bytes.length !== expectedLength) throw new Error(`Invalid ${label} length.`);
  return bytes;
}

function canonicalJson(value) {
  if (value === null || typeof value !== 'object') return JSON.stringify(value);
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(',')}]`;
  return `{${Object.keys(value).sort().map(key => `${JSON.stringify(key)}:${canonicalJson(value[key])}`).join(',')}}`;
}

function normalizeAgeArmor(value) {
  const text = value.trim();
  const header = '-----BEGIN AGE ENCRYPTED FILE-----';
  const footer = '-----END AGE ENCRYPTED FILE-----';
  if (!text.startsWith(header) || !text.endsWith(footer)) return text;
  const body = text.slice(header.length, -footer.length).replace(/\s/g, '');
  if (!body || body.length % 4 !== 0 || !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(body)) return text;
  const lines = body.match(/.{1,64}/g);
  if (!lines || lines.slice(0, -1).some(line => line.length !== 64)) return text;
  return `${header}\n${lines.join('\n')}\n${footer}`;
}

export async function publicKeyFingerprint(publicKey) {
  const key = publicKey instanceof Uint8Array ? publicKey : fromBase64(publicKey, ml_dsa65.lengths.publicKey, 'ML-DSA public key');
  const input = new Uint8Array(ROOT_DOMAIN.length + key.length);
  input.set(ROOT_DOMAIN);
  input.set(key, ROOT_DOMAIN.length);
  try {
    return [...sha256(input)].map(byte => byte.toString(16).padStart(2, '0')).join('');
  } finally {
    input.fill(0);
  }
}

export async function createEncryptedRootBackup(ageRecipient) {
  if (typeof ageRecipient !== 'string' || !ageRecipient.startsWith('age1pq1')) throw new Error('Unlock the instance age identity before creating its signing root.');
  const seed = crypto.getRandomValues(new Uint8Array(ml_dsa65.lengths.seed));
  let secretKey;
  let publicKey;
  let plaintext;
  let ciphertext;
  try {
    ({ secretKey, publicKey } = ml_dsa65.keygen(seed));
    const root = {
      format: 'rawrarr-encrypted-instance-root',
      version: ROOT_BACKUP_VERSION,
      algorithm: 'ML-DSA-65',
      publicKey: toBase64(publicKey),
      secretKey: toBase64(secretKey),
    };
    plaintext = encoder.encode(canonicalJson(root));
    const encryptor = new Encrypter();
    encryptor.addRecipient(ageRecipient);
    ciphertext = await encryptor.encrypt(plaintext);
    const encryptedBackup = armor.encode(ciphertext);
    return {
      algorithm: root.algorithm,
      publicKey: root.publicKey,
      fingerprint: await publicKeyFingerprint(publicKey),
      encryptedBackup,
    };
  } finally {
    seed.fill(0);
    secretKey?.fill(0);
    publicKey?.fill(0);
    plaintext?.fill(0);
    ciphertext?.fill(0);
  }
}

export async function inspectEncryptedRootBackup(encryptedBackup, ageIdentity) {
  if (typeof encryptedBackup !== 'string' || !encryptedBackup.trim().startsWith('-----BEGIN AGE ENCRYPTED FILE-----')) throw new Error('Paste an encrypted root-key backup.');
  if (typeof ageIdentity !== 'string' || !ageIdentity.trim().startsWith('AGE-SECRET-KEY-PQ-1')) throw new Error('Import the matching age identity first.');
  let ciphertext;
  let plaintext;
  let root;
  let secretKey;
  let publicKey;
  try {
    ciphertext = armor.decode(normalizeAgeArmor(encryptedBackup));
    const decryptor = new Decrypter();
    decryptor.addIdentity(ageIdentity.trim());
    plaintext = await decryptor.decrypt(ciphertext);
    root = JSON.parse(decoder.decode(plaintext));
    if (root.format !== 'rawrarr-encrypted-instance-root' || root.version !== ROOT_BACKUP_VERSION || root.algorithm !== 'ML-DSA-65') throw new Error('Unsupported encrypted root-key backup format.');
    secretKey = fromBase64(root.secretKey, ml_dsa65.lengths.secretKey, 'root private key');
    publicKey = fromBase64(root.publicKey, ml_dsa65.lengths.publicKey, 'root public key');
    if (!ml_dsa65.getPublicKey(secretKey).every((byte, index) => byte === publicKey[index])) throw new Error('The encrypted root backup contains mismatched keys.');
    return { algorithm: root.algorithm, publicKey: root.publicKey, fingerprint: await publicKeyFingerprint(publicKey) };
  } catch (error) {
    if (error instanceof Error && /^(Paste|Import|Unsupported|The encrypted)/.test(error.message)) throw error;
    throw new Error('Could not unlock the encrypted root-key backup with this age identity.');
  } finally {
    ciphertext?.fill(0);
    plaintext?.fill(0);
    secretKey?.fill(0);
    publicKey?.fill(0);
    if (root && typeof root.secretKey === 'string') root.secretKey = '';
  }
}

async function signDevicePayload(encryptedBackup, ageIdentity, pinnedFingerprint, domain, payload) {
  if (typeof encryptedBackup !== 'string' || !encryptedBackup.startsWith('-----BEGIN AGE ENCRYPTED FILE-----')) throw new Error('Paste the saved encrypted root-key backup.');
  if (typeof ageIdentity !== 'string' || !ageIdentity.trim().startsWith('AGE-SECRET-KEY-PQ-1')) throw new Error('Unlock the instance age identity first.');
  if (!/^[0-9a-f]{64}$/.test(pinnedFingerprint || '')) throw new Error('The instance signing root is not initialized.');
  let ciphertext;
  let plaintext;
  let root;
  let secretKey;
  let publicKey;
  let message;
  try {
    ciphertext = armor.decode(normalizeAgeArmor(encryptedBackup));
    const decryptor = new Decrypter();
    decryptor.addIdentity(ageIdentity.trim());
    plaintext = await decryptor.decrypt(ciphertext);
    root = JSON.parse(decoder.decode(plaintext));
    if (root.format !== 'rawrarr-encrypted-instance-root' || root.version !== ROOT_BACKUP_VERSION || root.algorithm !== 'ML-DSA-65') throw new Error('Unsupported encrypted root-key backup format.');
    secretKey = fromBase64(root.secretKey, ml_dsa65.lengths.secretKey, 'root private key');
    publicKey = fromBase64(root.publicKey, ml_dsa65.lengths.publicKey, 'root public key');
    if (!ml_dsa65.getPublicKey(secretKey).every((byte, index) => byte === publicKey[index])) throw new Error('The encrypted root backup contains mismatched keys.');
    const actualFingerprint = await publicKeyFingerprint(publicKey);
    if (actualFingerprint !== pinnedFingerprint) throw new Error('This backup does not match the instance signing root.');
    const signedPayload = { domain, ...payload };
    message = encoder.encode(canonicalJson(signedPayload));
    const signature = ml_dsa65.sign(message, secretKey);
    return {
      algorithm: 'ML-DSA-65',
      rootPublicKey: toBase64(publicKey),
      rootFingerprint: actualFingerprint,
      signature: toBase64(signature),
      signedPayload,
    };
  } catch (error) {
    if (error instanceof Error && /^(Paste|Unlock|The instance|Unsupported|The encrypted|This backup)/.test(error.message)) throw error;
    throw new Error('Could not unlock or validate the encrypted root-key backup. Check the pasted backup and age identity.');
  } finally {
    ciphertext?.fill(0);
    plaintext?.fill(0);
    secretKey?.fill(0);
    publicKey?.fill(0);
    message?.fill(0);
    if (root && typeof root.secretKey === 'string') root.secretKey = '';
  }
}

export function signDeviceApproval(encryptedBackup, ageIdentity, pinnedFingerprint, approval) {
  return signDevicePayload(encryptedBackup, ageIdentity, pinnedFingerprint, APPROVAL_DOMAIN, approval);
}

async function decryptAge(identity, ciphertext) {
  const decryptor = new Decrypter();
  decryptor.addIdentity(identity.trim());
  return decryptor.decrypt(ciphertext);
}

function validateAppRegistry(registry) {
  if (!registry || typeof registry !== 'object' || Array.isArray(registry)
    || registry.schemaVersion !== 1 || !registry.apps || typeof registry.apps !== 'object'
    || Array.isArray(registry.apps) || Object.keys(registry.apps).length > 2) {
    throw new Error('The prepared app identity registry has an unsupported format.');
  }
  for (const [appType, app] of Object.entries(registry.apps)) {
    if (!['radarr', 'sonarr'].includes(appType) || !app || typeof app !== 'object'
      || typeof app.instanceId !== 'string' || !/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(app.instanceId)
      || typeof app.identity !== 'string' || !app.identity.trim().startsWith('AGE-SECRET-KEY-PQ-1')
      || typeof app.recipient !== 'string' || !app.recipient.startsWith('age1pq1')) {
      throw new Error('The prepared app identity registry contains an invalid app identity.');
    }
  }
}

/**
 * Decrypt the owner-wrapped group key and group-encrypted prepared app registry
 * in the owner client. The registry ciphertext alone is not authorization.
 */
export async function decryptOwnerAuthorizedAppIdentity(ownerAgeIdentity, ownerGroupEnvelope, encryptedRegistry, appType, appInstanceId) {
  if (typeof ownerAgeIdentity !== 'string' || !ownerAgeIdentity.trim().startsWith('AGE-SECRET-KEY-PQ-1')) {
    throw new Error('Unlock the owner age identity to authorize app-key issuance.');
  }
  if (!['radarr', 'sonarr'].includes(appType) || typeof appInstanceId !== 'string') {
    throw new Error('Select a valid managed app instance.');
  }
  if (!(ownerGroupEnvelope instanceof Uint8Array) || ownerGroupEnvelope.length === 0 || ownerGroupEnvelope.length > MAX_APP_REGISTRY_BYTES
    || !(encryptedRegistry instanceof Uint8Array) || encryptedRegistry.length === 0 || encryptedRegistry.length > MAX_APP_REGISTRY_BYTES) {
    throw new Error('The owner-wrapped app identity files are missing or oversized.');
  }
  let groupIdentityBytes;
  let groupIdentity = '';
  let registryBytes;
  let registry;
  try {
    groupIdentityBytes = await decryptAge(ownerAgeIdentity, ownerGroupEnvelope);
    groupIdentity = decoder.decode(groupIdentityBytes);
    if (!groupIdentity.trim().startsWith('AGE-SECRET-KEY-PQ-1')) throw new Error('The owner-wrapped group identity is invalid.');
    registryBytes = await decryptAge(groupIdentity, encryptedRegistry);
    if (registryBytes.length > MAX_APP_REGISTRY_BYTES) throw new Error('The prepared app identity registry is oversized.');
    registry = JSON.parse(decoder.decode(registryBytes));
    validateAppRegistry(registry);
    const app = registry.apps[appType];
    if (!app || app.instanceId !== appInstanceId) throw new Error('That app identity is unavailable for the selected instance.');
    const actualRecipient = await identityToRecipient(app.identity);
    if (actualRecipient !== app.recipient) throw new Error('The prepared app identity does not match its public recipient.');
    return { appInstanceId: app.instanceId, appType, appIdentity: app.identity, appRecipient: app.recipient };
  } catch (error) {
    if (error instanceof Error && /^(Unlock|Select|The owner-wrapped|The prepared|That app|The prepared app)/.test(error.message)) throw error;
    throw new Error('Could not authorize app-key issuance with the owner age identity.');
  } finally {
    groupIdentityBytes?.fill(0);
    groupIdentity = '';
    registryBytes?.fill(0);
    if (registry && typeof registry === 'object') {
      for (const app of Object.values(registry.apps || {})) if (app && typeof app.identity === 'string') app.identity = '';
    }
  }
}

function validateAppGrantInput({ appInstanceId, appType, appRecipient, deviceId, deviceRecipient, keyEpoch, minimumKeyEpoch, appIdentity }) {
  if (typeof appInstanceId !== 'string' || !/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(appInstanceId)) {
    throw new Error('The app instance ID must be a canonical UUID.');
  }
  if (typeof appType !== 'string' || !/^[a-z0-9-]{1,64}$/.test(appType)) throw new Error('The app type is invalid.');
  if (typeof deviceId !== 'string' || !deviceId.trim() || deviceId.length > 128 || /[\u0000-\u001f\u007f]/.test(deviceId)) {
    throw new Error('Select a valid approved fossil.');
  }
  if (!Number.isSafeInteger(minimumKeyEpoch) || minimumKeyEpoch < 0) throw new Error('Refresh the target fossil’s accepted app-key epoch before issuing a grant.');
  if (!Number.isSafeInteger(keyEpoch) || keyEpoch < 1 || keyEpoch <= minimumKeyEpoch) {
    throw new Error(`The new app-key epoch must be greater than the target fossil’s accepted epoch (${minimumKeyEpoch}).`);
  }
  if (typeof deviceRecipient !== 'string' || !/^age1pq1[a-z0-9]+$/.test(deviceRecipient)
    || typeof appRecipient !== 'string' || !/^age1pq1[a-z0-9]+$/.test(appRecipient)) {
    throw new Error('The app or fossil age recipient is invalid.');
  }
  if (typeof appIdentity !== 'string' || !appIdentity.trim().startsWith('AGE-SECRET-KEY-PQ-1')) throw new Error('The owner-authorized app identity is unavailable.');
}

/** Create a canonical root-signed grant entirely in the owner browser. */
export async function createAppKeyGrant(encryptedBackup, ownerAgeIdentity, pinnedFingerprint, grantInput) {
  validateAppGrantInput(grantInput || {});
  const {
    appInstanceId, appType, appRecipient, deviceId, deviceRecipient, keyEpoch, appIdentity,
  } = grantInput;
  let plaintext;
  let envelope;
  try {
    if (await identityToRecipient(appIdentity) !== appRecipient) throw new Error('The app identity does not match the selected app recipient.');
    plaintext = encoder.encode(appIdentity.trim());
    if (plaintext.length > MAX_APP_IDENTITY_BYTES) throw new Error('The app identity is oversized.');
    const encrypter = new Encrypter();
    encrypter.addRecipient(deviceRecipient);
    envelope = new Uint8Array(await encrypter.encrypt(plaintext));
    const digest = [...sha256(envelope)].map(byte => byte.toString(16).padStart(2, '0')).join('');
    const signed = await signDevicePayload(encryptedBackup, ownerAgeIdentity, pinnedFingerprint, APP_KEY_GRANT_DOMAIN, {
      schemaVersion: 1,
      appInstanceId,
      appType,
      appRecipient,
      deviceId,
      deviceRecipient,
      keyEpoch,
      encryptedIdentitySha256: digest,
    });
    return {
      algorithm: 'ML-DSA-65',
      rootPublicKey: signed.rootPublicKey,
      rootFingerprint: signed.rootFingerprint,
      signedPayload: signed.signedPayload,
      signature: signed.signature,
      encryptedIdentity: toBase64(envelope),
    };
  } finally {
    plaintext?.fill(0);
    envelope?.fill(0);
  }
}

export async function signDeviceRejection(encryptedBackup, ageIdentity, pinnedFingerprint, rejection) {
  if (!rejection || typeof rejection !== 'object') throw new Error('Invalid device rejection details.');
  return signDevicePayload(encryptedBackup, ageIdentity, pinnedFingerprint, REVOCATION_DOMAIN, {
    requestId: rejection.requestId,
    nodeId: rejection.nodeId,
    role: rejection.role,
    displayName: rejection.displayName,
    publicKeyHex: rejection.publicKeyHex,
  });
}

export function verifyDeviceApproval(signatureBase64, publicKeyBase64, signedPayload) {
  const signature = fromBase64(signatureBase64, ml_dsa65.lengths.signature, 'ML-DSA signature');
  const publicKey = fromBase64(publicKeyBase64, ml_dsa65.lengths.publicKey, 'ML-DSA public key');
  const message = encoder.encode(canonicalJson(signedPayload));
  try {
    return ml_dsa65.verify(signature, message, publicKey);
  } finally {
    signature.fill(0);
    publicKey.fill(0);
    message.fill(0);
  }
}
