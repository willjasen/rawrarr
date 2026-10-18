import * as age from 'age-encryption';
import { zipSync } from 'fflate';

const encoder = new TextEncoder();
const decoder = new TextDecoder();

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
  try {
    const decrypter = new age.Decrypter();
    decrypter.addIdentity(identity.trim());
    return new Uint8Array(await decrypter.decrypt(ciphertext, type));
  } catch (_) {
    throw new Error('Could not decrypt this age file. Check the private ID and selected metadata.');
  }
}

async function fetchBytes(url) {
  const response = await fetch(url, { cache: 'no-store', headers: { Accept: 'application/octet-stream' } });
  if (!response.ok) throw new Error(`Could not fetch encrypted metadata (${response.status})`);
  return new Uint8Array(await response.arrayBuffer());
}

export async function runMiniExport(identity, onProgress = () => {}) {
  if (!identity.trim().startsWith('AGE-SECRET-KEY-PQ-1')) throw new Error('Paste a post-quantum age private ID.');
  onProgress('Reading encrypted metadata index…');
  const indexResponse = await fetch('/api/metadata/export/mini');
  if (!indexResponse.ok) throw new Error(indexResponse.status === 404 ? 'No encrypted movie metadata is available yet.' : 'Could not read the metadata export index.');
  const index = await indexResponse.json();
  if (!index.textFiles?.length) throw new Error('No encrypted movie text records are available for the mini export.');

  onProgress('Decrypting shared config in this browser…');
  const encryptedConfig = await fetchBytes('/api/metadata/export/config');
  const configBytes = await decrypt(identity, encryptedConfig);
  encryptedConfig.fill(0);
  let config;
  try { config = JSON.parse(decoder.decode(configBytes)); }
  finally { configBytes.fill(0); }
  let keyHex = config.metadata_filename_key;
  const validKey = /^[0-9a-f]{64}$/i.test(keyHex || '');
  for (const entry of Object.keys(config)) if (entry !== 'metadata_filename_key' && typeof config[entry] === 'string') config[entry] = '';
  if (!validKey) {
    keyHex = '';
    config.metadata_filename_key = '';
    config = null;
    throw new Error('The decrypted shared config has no valid metadata filename key.');
  }
  const filenameKey = Uint8Array.from(keyHex.match(/../g), (pair) => Number.parseInt(pair, 16));
  keyHex = '';
  config.metadata_filename_key = '';
  config.radarr_api_key = '';

  const files = {};
  try {
  const manifest = { formatVersion: 1, createdAt: new Date().toISOString(), scope: 'mini movie metadata export', movies: [] };
  const candidates = [];
  for (const [position, ciphertextName] of index.textFiles.entries()) {
    onProgress(`Decrypting movie record ${position + 1} of ${index.textFiles.length}…`);
    const encryptedText = await fetchBytes(`/api/metadata/export/ciphertext/text/${encodeURIComponent(ciphertextName)}`);
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
    const encrypted = await fetchBytes(`/api/metadata/export/ciphertext/artwork/${encodeURIComponent(artworkName)}`);
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
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = 'rawrarr-mini-metadata-export.zip';
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 60_000);
  return manifest.movies.length;
  } finally {
    for (const value of Object.values(files)) if (value instanceof Uint8Array) value.fill(0);
    filenameKey.fill(0);
    for (const entry of Object.keys(config || {})) if (typeof config[entry] === 'string') config[entry] = '';
    config = null;
  }
}

window.rawrarrMiniExport = { runMiniExport };
