import { Decrypter, Encrypter, armor, identityToRecipient } from 'age-encryption';
import { ml_dsa65 } from '@noble/post-quantum/ml-dsa.js';
import { sha256 } from '@noble/hashes/sha2.js';

const encoder = new TextEncoder();
const decoder = new TextDecoder('utf-8', { fatal: true });
const ROOT_BACKUP_VERSION = 1;
const ROOT_DOMAIN = encoder.encode('rawrarr-instance-root-v1\0');
const APPROVAL_DOMAIN = 'rawrarr-device-approval-v1';

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

export async function signDeviceApproval(encryptedBackup, ageIdentity, pinnedFingerprint, approval) {
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
    ciphertext = armor.decode(encryptedBackup.trim());
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
    const signedPayload = { domain: APPROVAL_DOMAIN, ...approval };
    message = encoder.encode(canonicalJson(signedPayload));
    const signature = ml_dsa65.sign(message, secretKey);
    return { algorithm: 'ML-DSA-65', rootFingerprint: actualFingerprint, signature: toBase64(signature), signedPayload };
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
