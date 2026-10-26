import * as age from 'age-encryption';
import { zipSync } from 'fflate';

const encoder = new TextEncoder();
const decoder = new TextDecoder();
let legacyUserIdentity = '';

function hex(bytes) {
  return [...bytes].map((value) => value.toString(16).padStart(2, '0')).join('');
}

async function movieFilename(key, kind, tmdbId, suffix) {
  const cryptoKey = await crypto.subtle.importKey('raw', key, { name: 'HMAC', hash: 'SHA-256' }, false, ['sign']);
  const input = encoder.encode(`rawrarr-metadata-filename-v1\0${kind}\0tmdb\0${tmdbId}`);
  const digest = new Uint8Array(await crypto.subtle.sign('HMAC', cryptoKey, input));
  return `${hex(digest)}${suffix}`;
}

async function decrypt(identity, ciphertext, type = 'uint8array') {
  const candidates = [identity.trim()];
  if (legacyUserIdentity && legacyUserIdentity !== candidates[0]) candidates.push(legacyUserIdentity);
  for (const candidate of candidates) {
    try {
      const decrypter = new age.Decrypter();
      decrypter.addIdentity(candidate);
      return new Uint8Array(await decrypter.decrypt(ciphertext, type));
    } catch (_) { /* Try legacy user-recipient ciphertext while migrating older shared files. */ }
  }
  throw new Error('Could not decrypt this age file. Check the private ID and selected metadata.');
}

async function fetchBytes(url) {
  const response = await fetch(url, { cache: 'no-store', credentials: 'include', headers: { Accept: 'application/octet-stream' } });
  if (!response.ok) throw new Error(`Could not fetch encrypted metadata (${response.status})`);
  return new Uint8Array(await response.arrayBuffer());
}

async function resolveGroupIdentity(userIdentity, base) {
  const prefix = base.endsWith('/') ? base : `${base}/`;
  const response = await fetch(`${prefix}api/metadata/export/group-user-envelope`, {
    cache: 'no-store', credentials: 'include', headers: { Accept: 'application/octet-stream' },
  });
  legacyUserIdentity = userIdentity.trim();
  if (response.status === 404) return userIdentity;
  if (!response.ok) throw new Error('Could not read the group identity envelope for this user.');
  const ciphertext = new Uint8Array(await response.arrayBuffer());
  const plaintext = await decrypt(userIdentity, ciphertext, 'text');
  ciphertext.fill(0);
  const groupIdentity = String(plaintext).trim();
  if (!groupIdentity.startsWith('AGE-SECRET-KEY-PQ-1')) throw new Error('The group identity envelope is invalid.');
  return groupIdentity;
}

async function encryptIdentityForDevice(identity, apiBase = '/api') {
  if (typeof identity !== 'string' || !identity.trim().startsWith('AGE-SECRET-KEY-PQ-1')) {
    throw new Error('Paste a post-quantum age private ID.');
  }
  const prefix = apiBase.replace(/\/$/, '');
  const response = await fetch(`${prefix}/metadata/export/node-recipient`, {
    cache: 'no-store', credentials: 'include', headers: { Accept: 'text/plain' },
  });
  if (!response.ok) throw new Error('Could not read this device’s public age recipient.');
  const recipient = (await response.text()).trim();
  const encrypter = new age.Encrypter();
  encrypter.addRecipient(recipient);
  const plaintext = encoder.encode(identity.trim());
  try {
    const ciphertext = new Uint8Array(await encrypter.encrypt(plaintext));
    let binary = '';
    for (let offset = 0; offset < ciphertext.length; offset += 0x8000) {
      binary += String.fromCharCode(...ciphertext.subarray(offset, offset + 0x8000));
    }
    ciphertext.fill(0);
    return btoa(binary);
  } finally { plaintext.fill(0); }
}

async function prepareGroupIdentityForDevice(userIdentity, apiBase = '/api') {
  legacyUserIdentity = userIdentity.trim();
  const prefix = apiBase.replace(/\/$/, '');
  const envelopeResponse = await fetch(`${prefix}/metadata/export/group-user-envelope`, {
    cache: 'no-store', credentials: 'include', headers: { Accept: 'application/octet-stream' },
  });
  if (envelopeResponse.status === 404) return false;
  if (!envelopeResponse.ok) throw new Error('Could not read the group identity envelope for this user.');
  const userEnvelope = new Uint8Array(await envelopeResponse.arrayBuffer());
  const groupIdentityBytes = new Uint8Array(await decrypt(userIdentity, userEnvelope));
  userEnvelope.fill(0);
  const groupIdentity = decoder.decode(groupIdentityBytes).trim();
  if (!groupIdentity.startsWith('AGE-SECRET-KEY-PQ-1')) {
    groupIdentityBytes.fill(0);
    throw new Error('The group identity envelope is invalid.');
  }
  const recipientResponse = await fetch(`${prefix}/metadata/export/node-recipient`, {
    cache: 'no-store', credentials: 'include', headers: { Accept: 'text/plain' },
  });
  if (!recipientResponse.ok) {
    groupIdentityBytes.fill(0);
    throw new Error('Could not read this device’s public age recipient.');
  }
  const encrypter = new age.Encrypter();
  encrypter.addRecipient((await recipientResponse.text()).trim());
  const ciphertext = new Uint8Array(await encrypter.encrypt(groupIdentityBytes));
  groupIdentityBytes.fill(0);
  try {
    const installed = await fetch(`${prefix}/settings/age/group-identity`, {
      method: 'PUT', cache: 'no-store', credentials: 'include',
      headers: { 'Content-Type': 'application/octet-stream' }, body: ciphertext,
    });
    if (!installed.ok) throw new Error(`Could not install the group identity on this device (${installed.status}).`);
  } finally { ciphertext.fill(0); }
  return true;
}

function normalizeConfigShape(config) {
  if (!Array.isArray(config.services)) {
    const services = [];
    if (config.radarr_url || config.radarr_api_key) services.push({ type: 'radarr', url: config.radarr_url || '', api_key: config.radarr_api_key || '' });
    if (config.sonarr && typeof config.sonarr === 'object') services.push({ type: 'sonarr', url: config.sonarr.url || '', api_key: config.sonarr.api_key || '' });
    config.services = services;
  } else {
    const known = new Set(config.services.map(service => service?.type));
    if ((config.radarr_url || config.radarr_api_key) && !known.has('radarr')) {
      config.services.push({ type: 'radarr', url: config.radarr_url || '', api_key: config.radarr_api_key || '' });
    }
    if (config.sonarr && typeof config.sonarr === 'object' && !known.has('sonarr')) {
      config.services.push({ type: 'sonarr', url: config.sonarr.url || '', api_key: config.sonarr.api_key || '' });
    }
  }
  delete config.radarr_url;
  delete config.radarr_api_key;
  delete config.sonarr;
  config.schema_version = 1;
  if (!config.device_names || typeof config.device_names !== 'object' || Array.isArray(config.device_names)) config.device_names = {};
  return config;
}

function canonicalizeConfig(config) {
  normalizeConfigShape(config);
  if (!/^[0-9a-f]{64}$/.test(config.metadata_filename_key || '')) {
    const randomKey = crypto.getRandomValues(new Uint8Array(32));
    config.metadata_filename_key = [...randomKey].map(byte => byte.toString(16).padStart(2, '0')).join('');
    randomKey.fill(0);
  }
  return config;
}

function clearConfigStrings(value) {
  if (Array.isArray(value)) {
    for (let index = 0; index < value.length; index++) {
      if (typeof value[index] === 'string') value[index] = '';
      else clearConfigStrings(value[index]);
    }
  } else if (value && typeof value === 'object') {
    for (const key of Object.keys(value)) {
      if (typeof value[key] === 'string') value[key] = '';
      else clearConfigStrings(value[key]);
    }
  }
}

export async function saveDeviceName(identity, base, nodeId, name) {
  if (!identity.trim().startsWith('AGE-SECRET-KEY-PQ-1')) throw new Error('Paste a post-quantum age private ID.');
  const prefix = base.endsWith('/') ? base : `${base}/`;
  identity = await resolveGroupIdentity(identity, base);
  const encrypted = await fetchBytes(`${prefix}api/metadata/export/config`);
  const plaintext = await decrypt(identity, encrypted);
  encrypted.fill(0);
  let config;
  try { config = JSON.parse(decoder.decode(plaintext)); }
  finally { plaintext.fill(0); }
  canonicalizeConfig(config);
  const names = config.device_names && typeof config.device_names === 'object' && !Array.isArray(config.device_names)
    ? config.device_names : {};
  if (name) names[nodeId] = name; else delete names[nodeId];
  config.device_names = names;
  const recipientResponse = await fetch(`${prefix}api/metadata/export/recipient`, { cache: 'no-store' });
  if (!recipientResponse.ok) throw new Error('Could not read the public age recipient from this fossil node.');
  const recipient = (await recipientResponse.text()).trim();
  const encrypter = new age.Encrypter();
  encrypter.addRecipient(recipient);
  const updated = new TextEncoder().encode(JSON.stringify(config));
  try {
    const ciphertext = new Uint8Array(await encrypter.encrypt(updated));
    const response = await fetch(`${prefix}api/metadata/export/config`, {
      method: 'PUT', cache: 'no-store', headers: { 'Content-Type': 'application/octet-stream' }, body: ciphertext
    });
    ciphertext.fill(0);
    if (!response.ok) throw new Error(`Could not save encrypted device name (${response.status}).`);
  } finally {
    updated.fill(0);
    clearConfigStrings(config);
    config = null;
  }
}

export async function loadDeviceNames(identity, base) {
  const prefix = base.endsWith('/') ? base : `${base}/`;
  identity = await resolveGroupIdentity(identity, base);
  const encrypted = await fetchBytes(`${prefix}api/metadata/export/config`);
  const plaintext = await decrypt(identity, encrypted);
  encrypted.fill(0);
  let config;
  try { config = JSON.parse(decoder.decode(plaintext)); }
  finally { plaintext.fill(0); }
  const names = config.device_names && typeof config.device_names === 'object' && !Array.isArray(config.device_names)
    ? { ...config.device_names } : {};
  clearConfigStrings(config);
  config = null;
  return names;
}

export async function loadSharedConfig(identity, base) {
  if (!identity.trim().startsWith('AGE-SECRET-KEY-PQ-1')) throw new Error('Paste a post-quantum age private ID.');
  const prefix = base.endsWith('/') ? base : `${base}/`;
  identity = await resolveGroupIdentity(identity, base);
  const encrypted = await fetchBytes(`${prefix}api/metadata/export/config`);
  const plaintext = await decrypt(identity, encrypted);
  encrypted.fill(0);
  try {
    const config = JSON.parse(decoder.decode(plaintext));
    if (!config || typeof config !== 'object' || Array.isArray(config)) throw new Error('The shared configuration has an invalid format.');
    return normalizeConfigShape(config);
  } catch (error) {
    if (error instanceof SyntaxError) throw new Error('The decrypted shared configuration is not valid JSON.');
    throw error;
  } finally { plaintext.fill(0); }
}

export async function runMiniExport(identity, onProgress = () => {}, base = '') {
  const endpointUrl = path => `${base}${path}`;
  if (!identity.trim().startsWith('AGE-SECRET-KEY-PQ-1')) throw new Error('Paste a post-quantum age private ID.');
  identity = await resolveGroupIdentity(identity, base);
  onProgress('Reading encrypted metadata index…');
  const indexResponse = await fetch(endpointUrl('/api/metadata/export/mini'), { cache: 'no-store' });
  if (!indexResponse.ok) throw new Error(indexResponse.status === 404 ? 'No encrypted movie metadata is available yet.' : 'Could not read the metadata export index.');
  const index = await indexResponse.json();
  if (!index.textFiles?.length) throw new Error('No encrypted movie metadata records are available to export.');

  onProgress('Decrypting shared config in this browser…');
  const encryptedConfig = await fetchBytes(endpointUrl('/api/metadata/export/config'));
  const configBytes = await decrypt(identity, encryptedConfig);
  encryptedConfig.fill(0);
  let config;
  try { config = JSON.parse(decoder.decode(configBytes)); }
  finally { configBytes.fill(0); }
  let keyHex = config.metadata_filename_key;
  const validKey = /^[0-9a-f]{64}$/i.test(keyHex || '');
  clearConfigStrings(config);
  if (!validKey) {
    keyHex = '';
    config.metadata_filename_key = '';
    config = null;
    throw new Error('The decrypted shared config has no valid metadata filename key.');
  }
  const filenameKey = Uint8Array.from(keyHex.match(/../g), (pair) => Number.parseInt(pair, 16));
  keyHex = '';
  config.metadata_filename_key = '';

  const files = {};
  try {
  const manifest = { formatVersion: 1, createdAt: new Date().toISOString(), scope: 'all available movie metadata and matching artwork', movies: [] };
  const candidates = [];
  for (const [position, ciphertextName] of index.textFiles.entries()) {
    onProgress(`Decrypting movie record ${position + 1} of ${index.textFiles.length}…`);
    const encryptedText = await fetchBytes(endpointUrl(`/api/metadata/export/ciphertext/text/${encodeURIComponent(ciphertextName)}`));
    const textBytes = await decrypt(identity, encryptedText);
    encryptedText.fill(0);
    const record = JSON.parse(decoder.decode(textBytes));
    const tmdbId = Number(record.tmdbId);
    if (!Number.isSafeInteger(tmdbId) || tmdbId <= 0) throw new Error('A decrypted movie record has an invalid TMDb ID.');
    const expectedTextName = await movieFilename(filenameKey, 'text', tmdbId, '.json.aged');
    if (ciphertextName !== expectedTextName) throw new Error('A movie record filename did not match its decrypted TMDb ID.');
    const textPath = `movies/text/${tmdbId}.json`;
    files[textPath] = textBytes;
    candidates.push({ tmdbId, record, textPath, textCiphertextPath: `movies/text/${ciphertextName}` });
  }

  for (const [position, movie] of candidates.entries()) {
    onProgress(`Matching artwork ${position + 1} of ${candidates.length}…`);
    const artworkName = await movieFilename(filenameKey, 'artwork', movie.tmdbId, '.aged');
    if (!index.artworkFiles.includes(artworkName)) continue;
    const encrypted = await fetchBytes(endpointUrl(`/api/metadata/export/ciphertext/artwork/${encodeURIComponent(artworkName)}`));
    const wrappedArtwork = await decrypt(identity, encrypted);
    encrypted.fill(0);
    const separator = wrappedArtwork.indexOf(10);
    if (separator < 1) throw new Error(`Artwork for TMDb ${movie.tmdbId} has an invalid content-type header.`);
    const contentType = decoder.decode(wrappedArtwork.subarray(0, separator));
    const extension = ({ 'image/jpeg': 'jpg', 'image/png': 'png', 'image/webp': 'webp' })[contentType];
    if (!extension) throw new Error(`Artwork for TMDb ${movie.tmdbId} has an unsupported content type.`);
    const plaintext = wrappedArtwork.slice(separator + 1);
    wrappedArtwork.fill(0);
    const artworkPath = `movies/artwork/${movie.tmdbId}.${extension}`;
    files[artworkPath] = plaintext;
    movie.artworkPath = artworkPath;
    movie.artworkCiphertextPath = `movies/artwork/${artworkName}`;
  }

  manifest.movies = candidates.map(({ tmdbId, record, textPath, textCiphertextPath, artworkPath, artworkCiphertextPath }) => ({
    tmdbId,
    title: typeof record.title === 'string' ? record.title : null,
    textPath,
    textCiphertextPath,
    artworkPath: artworkPath || null,
    artworkCiphertextPath: artworkCiphertextPath || null,
  }));
  files['manifest.json'] = encoder.encode(JSON.stringify(manifest, null, 2));
  onProgress('Building the ZIP in this browser…');
  const archive = zipSync(files, { level: 0 });
  const blob = new Blob([archive], { type: 'application/zip' });
  const downloadUrl = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = downloadUrl;
  link.download = 'rawrarr-metadata-export.zip';
  link.click();
  setTimeout(() => URL.revokeObjectURL(downloadUrl), 60_000);
  return manifest.movies.length;
  } finally {
    for (const value of Object.values(files)) if (value instanceof Uint8Array) value.fill(0);
    filenameKey.fill(0);
    for (const entry of Object.keys(config || {})) if (typeof config[entry] === 'string') config[entry] = '';
    config = null;
  }
}

export async function loadMiniMetadata(identity, base = '', onProgress = () => {}) {
  if (!identity.trim().startsWith('AGE-SECRET-KEY-PQ-1')) throw new Error('Paste a post-quantum age private ID.');
  identity = await resolveGroupIdentity(identity, base);
  const url = path => `${base}${path}`;
  onProgress('Reading encrypted metadata index…');
  const response = await fetch(url('/api/metadata/export/mini'), { cache: 'no-store' });
  if (!response.ok) throw new Error(response.status === 404 ? 'No encrypted movie metadata is available yet.' : `Could not read metadata (${response.status}).`);
  const index = await response.json();
  if (!index.textFiles?.length) return [];
  onProgress('Decrypting shared config in this browser…');
  const encryptedConfig = await fetchBytes(url('/api/metadata/export/config'));
  const configBytes = await decrypt(identity, encryptedConfig);
  encryptedConfig.fill(0);
  let config;
  try { config = JSON.parse(decoder.decode(configBytes)); } finally { configBytes.fill(0); }
  let keyHex = config.metadata_filename_key;
  if (!/^[0-9a-f]{64}$/i.test(keyHex || '')) throw new Error('The encrypted config has no valid metadata filename key.');
  const filenameKey = Uint8Array.from(keyHex.match(/../g), pair => Number.parseInt(pair, 16));
  keyHex = '';
  clearConfigStrings(config);
  config = null;
  const movies = [];
  try {
    for (const [position, name] of index.textFiles.entries()) {
      onProgress(`Decrypting movie record ${position + 1} of ${index.textFiles.length}…`);
      const encrypted = await fetchBytes(url(`/api/metadata/export/ciphertext/text/${encodeURIComponent(name)}`));
      const bytes = await decrypt(identity, encrypted); encrypted.fill(0);
      let record;
      try { record = JSON.parse(decoder.decode(bytes)); } finally { bytes.fill(0); }
      const tmdbId = Number(record.tmdbId);
      if (!Number.isSafeInteger(tmdbId) || tmdbId <= 0 || name !== await movieFilename(filenameKey, 'text', tmdbId, '.json.aged')) throw new Error('A movie record failed its encrypted identity check.');
      const movie = { tmdbId, title: record.title || 'Untitled', year: record.year || null, overview: record.overview || '', genres: Array.isArray(record.genres) ? record.genres : [], artworkUrl: null };
      const artworkName = await movieFilename(filenameKey, 'artwork', tmdbId, '.aged');
      if (index.artworkFiles.includes(artworkName)) {
        const encryptedArt = await fetchBytes(url(`/api/metadata/export/ciphertext/artwork/${encodeURIComponent(artworkName)}`));
        const wrapped = await decrypt(identity, encryptedArt); encryptedArt.fill(0);
        const separator = wrapped.indexOf(10);
        if (separator > 0) {
          const type = decoder.decode(wrapped.subarray(0, separator));
          if (['image/jpeg', 'image/png', 'image/webp'].includes(type)) movie.artworkUrl = URL.createObjectURL(new Blob([wrapped.slice(separator + 1)], { type }));
        }
        wrapped.fill(0);
      }
      movies.push(movie);
    }
    return movies;
  } finally { filenameKey.fill(0); }
}

export async function loadRadarrLibrary(identity, base = '', onProgress = () => {}) {
  if (!identity.trim().startsWith('AGE-SECRET-KEY-PQ-1')) throw new Error('Paste a post-quantum age private ID.');
  identity = await resolveGroupIdentity(identity, base);
  const url = path => `${base}${path}`;
  onProgress('Fetching encrypted Radarr library…');
  const keyPair = await crypto.subtle.generateKey({ name: 'ECDH', namedCurve: 'P-256' }, true, ['deriveBits']);
  const browserPublicKey = new Uint8Array(await crypto.subtle.exportKey('raw', keyPair.publicKey));
  const libraryResponse = await fetch(url('/api/metadata/session/library'), {
    method: 'POST', cache: 'no-store',
    headers: { 'Content-Type': 'application/octet-stream', Accept: 'application/octet-stream' },
    body: browserPublicKey,
  });
  browserPublicKey.fill(0);
  if (!libraryResponse.ok) throw new Error(`Could not establish the encrypted library session (${libraryResponse.status}).`);
  const encryptedLibrary = new Uint8Array(await libraryResponse.arrayBuffer());
  const nodePublicKey = await crypto.subtle.importKey('raw', encryptedLibrary.subarray(0, 65), { name: 'ECDH', namedCurve: 'P-256' }, false, []);
  const sharedSecret = new Uint8Array(await crypto.subtle.deriveBits({ name: 'ECDH', public: nodePublicKey }, keyPair.privateKey, 256));
  const hkdfSalt = new TextEncoder().encode('rawrarr-coordinator-session-v1');
  const hkdfInfo = new TextEncoder().encode('radarr-library-response');
  let sessionKey;
  try {
    const hkdfMaterial = await crypto.subtle.importKey('raw', sharedSecret, 'HKDF', false, ['deriveKey']);
    sessionKey = await crypto.subtle.deriveKey(
      { name: 'HKDF', hash: 'SHA-256', salt: hkdfSalt, info: hkdfInfo },
      hkdfMaterial,
      { name: 'AES-GCM', length: 256 },
      false,
      ['decrypt'],
    );
  } finally {
    sharedSecret.fill(0);
    hkdfSalt.fill(0);
    hkdfInfo.fill(0);
  }
  const iv = encryptedLibrary.slice(65, 77);
  const encryptedPayload = encryptedLibrary.slice(77);
  encryptedLibrary.fill(0);
  let libraryBytes;
  try { libraryBytes = new Uint8Array(await crypto.subtle.decrypt({ name: 'AES-GCM', iv }, sessionKey, encryptedPayload)); }
  catch (_) { throw new Error('The encrypted Radarr library response failed authentication.'); }
  finally { iv.fill(0); encryptedPayload.fill(0); }
  let library;
  try { library = JSON.parse(decoder.decode(libraryBytes)); }
  finally { libraryBytes.fill(0); }
  if (!Array.isArray(library)) throw new Error('The decrypted Radarr library has an invalid format.');

  onProgress('Decrypting shared config and reading encrypted artwork index…');
  const encryptedConfig = await fetchBytes(url('/api/metadata/export/config'));
  const configBytes = await decrypt(identity, encryptedConfig);
  encryptedConfig.fill(0);
  let config;
  try { config = JSON.parse(decoder.decode(configBytes)); }
  finally { configBytes.fill(0); }
  let keyHex = config.metadata_filename_key;
  if (!/^[0-9a-f]{64}$/i.test(keyHex || '')) throw new Error('The encrypted config has no valid metadata filename key.');
  const filenameKey = Uint8Array.from(keyHex.match(/../g), pair => Number.parseInt(pair, 16));
  keyHex = '';
  clearConfigStrings(config);
  config = null;
  try {
    const indexResponse = await fetch(url('/api/metadata/export/mini'), { cache: 'no-store' });
    const index = indexResponse.ok ? await indexResponse.json() : { artworkFiles: [] };
    const artworkFiles = new Set(index.artworkFiles || []);
    const movies = [];
    for (const [position, item] of library.entries()) {
      const tmdbId = Number(item.tmdbId);
      const movie = {
        title: typeof item.title === 'string' ? item.title : 'Unknown title',
        year: Number.isSafeInteger(item.year) ? item.year : null,
        monitored: Boolean(item.monitored),
        hasFile: Boolean(item.hasFile),
        sizeOnDisk: Number(item.sizeOnDisk) || 0,
        qualityProfile: typeof item.qualityProfile === 'string' ? item.qualityProfile : '—',
        added: item.added || null,
        tmdbId: Number.isSafeInteger(tmdbId) && tmdbId > 0 ? tmdbId : null,
        artworkSource: item.artworkSource || null,
        genres: Array.isArray(item.genres) ? item.genres.filter(genre => typeof genre === 'string') : [],
        artworkUrl: null,
      };
      if (movie.tmdbId) {
        const artworkName = await movieFilename(filenameKey, 'artwork', movie.tmdbId, '.aged');
        if (artworkFiles.has(artworkName)) movie.loadArtwork = async () => {
          const encrypted = await fetchBytes(url(`/api/metadata/export/ciphertext/artwork/${encodeURIComponent(artworkName)}`));
          const wrapped = await decrypt(identity, encrypted);
          encrypted.fill(0);
          const separator = wrapped.indexOf(10);
          if (separator < 1) { wrapped.fill(0); throw new Error('Encrypted artwork has an invalid content-type header.'); }
          const type = decoder.decode(wrapped.subarray(0, separator));
          if (!['image/jpeg', 'image/png', 'image/webp'].includes(type)) { wrapped.fill(0); throw new Error('Encrypted artwork has an unsupported content type.'); }
          const blob = new Blob([wrapped.slice(separator + 1)], { type });
          wrapped.fill(0);
          return URL.createObjectURL(blob);
        };
      }
      movies.push(movie);
      if ((position + 1) % 100 === 0) onProgress(`Decrypting movie ${position + 1} of ${library.length}…`);
    }
    return movies;
  } finally { filenameKey.fill(0); }
}

async function deriveProfileRecipient(identity) {
  if (typeof identity !== 'string' || !identity.trim()) throw new Error('No saved age ID is available.');
  return age.identityToRecipient(identity.trim());
}

window.rawrarrMiniExport = { runMiniExport, loadMiniMetadata, loadRadarrLibrary, saveDeviceName, loadDeviceNames, loadSharedConfig, deriveProfileRecipient, encryptIdentityForDevice, prepareGroupIdentityForDevice };
