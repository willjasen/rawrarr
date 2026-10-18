import assert from 'node:assert/strict';
import { createHmac, randomBytes } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { createServer } from 'node:http';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';
import { unzipSync } from '../../rust-arr/web-export/node_modules/fflate/esm/index.mjs';

const bundle = await (await import('node:fs/promises')).readFile(new URL('../../rust-arr/src/assets/metadata-export.js', import.meta.url));
const ageModule = await import('../../rust-arr/web-export/node_modules/age-encryption/dist/index.js');
const generatedIdentity = execFileSync('age-keygen', ['-pq'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
const privateIdentity = generatedIdentity.split('\n').find(line => line.startsWith('AGE-SECRET-KEY-PQ-1'));
assert.ok(privateIdentity);
const recipient = execFileSync('age-keygen', ['-y'], { input: privateIdentity, encoding: 'utf8' }).trim();
const filenameKey = randomBytes(32);
const records = [
  { tmdbId: 24001, title: 'Mini Export Fixture One', schemaVersion: 1 },
  { tmdbId: 24002, title: 'Mini Export Fixture Two', schemaVersion: 1 },
  { tmdbId: 24003, title: 'Mini Export Fixture Three', schemaVersion: 1 },
];
function hmacFilename(kind, id, suffix) {
  return `${createHmac('sha256', filenameKey).update(`rawrarr-metadata-filename-v1\0${kind}\0tmdb\0${id}`).digest('hex')}${suffix}`;
}
function encrypt(plaintext) {
  return execFileSync('age', ['--recipient', recipient], { input: plaintext });
}
const configCiphertext = encrypt(Buffer.from(JSON.stringify({ metadata_filename_key: filenameKey.toString('hex'), radarr_api_key: 'fixture-secret' })));
const encryptedText = new Map(records.map(record => [hmacFilename('text', record.tmdbId, '.json.aged'), encrypt(Buffer.from(JSON.stringify(record)))]));
const artworkPayloads = new Map(records.map((record, index) => {
  const bytes = Buffer.from([0xff, 0xd8, 0xff, index + 1, 0xff, 0xd9]);
  return [hmacFilename('artwork', record.tmdbId, '.aged'), { bytes, ciphertext: encrypt(Buffer.concat([Buffer.from('image/jpeg\n'), bytes])) }];
}));

let server;
let browser;
test('browser mini-export decrypts real post-quantum age files and builds a local ZIP', async t => {
  const tmp = await mkdtemp(join(tmpdir(), 'rawrarr-mini-export-'));
  t.after(async () => { await browser?.close(); await new Promise(resolve => server?.close(resolve)); await rm(tmp, { recursive: true, force: true }); });
  const requests = [];
  server = createServer((request, response) => {
    requests.push({ url: request.url, method: request.method, body: '' });
    if (request.url === '/') {
      response.setHeader('content-type', 'text/html');
      response.end('<button id="start">Start</button><input id="identity"><div id="status"></div><script src="/assets/metadata-export.js"></script><script>document.getElementById("start").onclick=async()=>{try{await window.rawrarrMiniExport.runMiniExport(document.getElementById("identity").value, s=>document.getElementById("status").textContent=s);document.getElementById("status").textContent="complete"}catch(e){document.getElementById("status").textContent=e.message}}</script>');
      return;
    }
    if (request.url === '/assets/metadata-export.js') {
      response.setHeader('content-type', 'text/javascript'); response.end(bundle); return;
    }
    if (request.url === '/api/metadata/export/mini') {
      response.setHeader('content-type', 'application/json');
      response.end(JSON.stringify({ textFiles: [...encryptedText.keys()], artworkFiles: [...artworkPayloads.keys()] })); return;
    }
    if (request.url === '/api/metadata/export/config') { response.end(configCiphertext); return; }
    const match = request.url.match(/^\/api\/metadata\/export\/ciphertext\/(text|artwork)\/([a-f0-9]{64}\.(?:json\.)?aged)$/);
    if (!match) { response.statusCode = 404; response.end(); return; }
    const [, kind, name] = match;
    const bytes = kind === 'text' ? encryptedText.get(name) : artworkPayloads.get(name)?.ciphertext;
    if (!bytes) { response.statusCode = 404; response.end(); return; }
    response.setHeader('cache-control', 'private, no-store'); response.end(bytes);
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address();
  browser = await chromium.launch({ headless: true, executablePath: '/usr/bin/chromium', args: ['--no-sandbox'] });
  const context = await browser.newContext({ acceptDownloads: true, downloadsPath: tmp });
  const page = await context.newPage();
  await page.goto(`http://127.0.0.1:${port}/`);

  const badIdentity = `${privateIdentity.slice(0, -1)}${privateIdentity.endsWith('A') ? 'B' : 'A'}`;
  await page.locator('#identity').fill(badIdentity);
  await page.locator('#start').click();
  await page.waitForFunction(() => document.getElementById('status').textContent.length > 0);
  assert.match(await page.locator('#status').textContent(), /decrypt/i);
  assert.equal((await page.locator('#status').textContent()).includes(privateIdentity), false);
  assert.equal(requests.some(({ url }) => url.includes('AGE-SECRET-KEY')), false);

  const downloadPromise = page.waitForEvent('download');
  await page.locator('#identity').fill(privateIdentity);
  await page.locator('#start').click();
  const download = await downloadPromise;
  assert.equal(download.suggestedFilename(), 'rawrarr-mini-metadata-export.zip');
  const archivePath = join(tmp, 'result.zip');
  await download.saveAs(archivePath);
  await page.waitForFunction(() => document.getElementById('status').textContent === 'complete');
  const archive = unzipSync(await (await import('node:fs/promises')).readFile(archivePath));
  const manifest = JSON.parse(Buffer.from(archive['manifest.json']).toString('utf8'));
  assert.equal(manifest.movies.length, 3);
  for (const record of records) {
    const text = JSON.parse(Buffer.from(archive[`movies/text/${record.tmdbId}.json`]).toString('utf8'));
    assert.equal(text.title, record.title);
    const movie = manifest.movies.find(item => item.tmdbId === record.tmdbId);
    assert.equal(movie.artworkPath, `movies/artwork/${record.tmdbId}.jpg`);
    const expected = artworkPayloads.get(hmacFilename('artwork', record.tmdbId, '.aged')).bytes;
    assert.deepEqual(Buffer.from(archive[movie.artworkPath]), expected);
  }
  assert.equal(requests.some(({ method, url, body }) => method !== 'GET' || url.includes(privateIdentity) || body.includes(privateIdentity)), false);
  assert.equal(await page.locator('#identity').inputValue(), privateIdentity);
  await context.close();
  assert.ok(await ageModule.identityToRecipient(privateIdentity));
});
