// Real services and UI only. The transparent observer never substitutes an application route.
import assert from 'node:assert/strict';
import { createHmac, randomBytes } from 'node:crypto';
import { execFileSync, spawn } from 'node:child_process';
import { once } from 'node:events';
import { createServer, request } from 'node:http';
import { mkdir, mkdtemp, open, readFile, rm, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { chromium } from '/usr/local/lib/node_modules/playwright-core/index.mjs';
import { unzipSync } from '../../../rust-arr/web-export/node_modules/fflate/esm/index.mjs';

assert.equal(process.env.RAWARR_BROWSER_EXPORT_TEST, 'disposable');
const auditPath = '/tmp/rawrarr-browser-export-audit.json';
function assertPrivate(bytes, markers, boundary) {
  const buffer = Buffer.from(bytes);
  for (const marker of markers) {
    for (const encoded of [marker, encodeURIComponent(marker), Buffer.from(marker).toString('base64'), Buffer.from(marker).toString('hex')]) {
      if (buffer.includes(Buffer.from(encoded))) throw new Error(`private fixture marker exposed at ${boundary}`);
    }
  }
}
const auditCanary = `audit canary:${randomBytes(24).toString('hex')}`;
for (const value of [auditCanary, encodeURIComponent(auditCanary), Buffer.from(auditCanary).toString('base64'), Buffer.from(auditCanary).toString('hex')]) {
  assert.throws(() => assertPrivate(Buffer.from(value), [auditCanary], 'self-test'), /private fixture marker exposed/);
}
assertPrivate(Buffer.from('public fixture control'), [auditCanary], 'self-test');
if (process.argv.includes('--audit-database')) {
  try {
    const chunks = [];
    for await (const chunk of process.stdin) chunks.push(chunk);
    const dump = Buffer.concat(chunks);
    const { markers } = JSON.parse(await readFile(auditPath, 'utf8'));
    assertPrivate(dump, markers, 'coordinator database');
    assert.ok(dump.includes(Buffer.from('rawrarr_passkeys')), 'database audit must include real auth tables');
    console.log('PASS coordinator PostgreSQL data dump contains none of the generated private markers');
  } finally {
    await rm(auditPath, { force: true });
  }
} else {
  await exercise();
}

async function exercise() {
  const root = await mkdtemp('/tmp/rawrarr-browser-export-');
  const origin = 'http://localhost:38080';
  const nodeId = 'export-browser-metadata';
  const relay = `/nodes/${nodeId}/api/metadata/export`;
  const children = [];
  const observed = [];
  const exchanges = [];
  const browserRequests = [];
  const records = [];
  const markers = [];
  let browser;
  let observer;
  let observerError;
  let observedBytes = 0;
  let stage = 'starting';
  const remember = value => { markers.push(value); return value; };
  const publicKey = identity => execFileSync('age-keygen', ['-y'], { input: identity, encoding: 'utf8', stdio: ['pipe', 'pipe', 'ignore'] }).trim();
  const identity = () => execFileSync('age-keygen', ['-pq'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] })
    .split('\n').find(line => line.startsWith('AGE-SECRET-KEY-PQ-1'));
  const encrypt = (recipient, bytes) => execFileSync('age', ['-r', recipient], { input: bytes, stdio: ['pipe', 'pipe', 'ignore'] });
  const record = bytes => {
    observedBytes += bytes.length;
    if (observedBytes > 64 * 1024 * 1024) throw new Error('coordinator observer capture limit exceeded');
    observed.push(Buffer.from(bytes));
  };
  async function start(role, port, extra = {}) {
    const directory = join(root, role);
    await mkdir(directory, { recursive: true });
    const logPath = join(directory, 'service.log');
    const log = await open(logPath, 'w', 0o600);
    const child = spawn('/opt/rawrarr-test-target/debug/rust-arr', [], {
      cwd: directory,
      env: {
        PATH: process.env.PATH, HOME: directory, RAWARR_ROLE: role,
        RAWARR_QUIET_LOGS: 'true', RUST_ARR_LISTEN_ADDR: `127.0.0.1:${port}`,
        ...extra,
      },
      stdio: ['ignore', log.fd, log.fd],
    });
    await log.close();
    children.push({ child, logPath });
    child.on('error', () => { observerError = new Error(`${role} process could not start`); });
    return child;
  }
  async function ready(child, port) {
    for (let attempt = 0; attempt < 300; attempt++) {
      assert.equal(child.exitCode, null, 'service exited before readiness');
      assert.equal(child.signalCode, null, 'service was killed before readiness');
      try {
        if ((await fetch(`http://127.0.0.1:${port}/health`)).ok) return;
      } catch (error) {
        if (error.cause?.code !== 'ECONNREFUSED') throw error;
      }
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    throw new Error('service readiness timeout');
  }
  async function stopServices() {
    for (const { child } of [...children].reverse()) {
      if (child.pid && child.exitCode === null && child.signalCode === null) {
        const stopped = once(child, 'exit');
        child.kill('SIGKILL');
        await stopped;
      }
    }
  }
  async function layout(page, selector) {
    const element = page.locator(selector);
    await element.scrollIntoViewIfNeeded();
    const box = await element.boundingBox();
    const viewport = page.viewportSize();
    assert.ok(box && box.width > 0 && box.height > 0 && box.x >= 0 && box.x + box.width <= viewport.width + 1,
      'primary action must fit the viewport');
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false,
      'page must not overflow horizontally');
  }
  try {
    await writeFile(join(root, 'registry.json'), '[]', { mode: 0o600 });
    stage = 'coordinator startup';
    const coordinator = await start('coordinator', 38079, {
      RAWARR_PG_HOST: '127.0.0.1', RAWARR_PG_USER: 'rawrarr',
      RAWARR_PG_DATABASE: 'rawrarr_browser_export_test', RAWARR_PG_PASSWORD: process.env.RAWARR_PG_PASSWORD,
      RAWARR_APPROVED_NODES_FILE: join(root, 'registry.json'),
      RAWARR_COORDINATOR_WEB_RP_ID: 'localhost', RAWARR_COORDINATOR_WEB_ORIGIN: origin,
      RAWARR_NODE_RELAY_TARGETS: JSON.stringify({ [nodeId]: 'http://127.0.0.1:38081' }),
      RAWARR_NODE_DIRECT_TARGETS: '{}', RAWARR_BILLING_AUDIENCE: 'internal',
    });
    await ready(coordinator, 38079);
    observer = createServer(async (req, res) => {
      try {
        record(Buffer.from(`${req.method} ${req.url}\n${JSON.stringify(req.headers)}`));
        const chunks = [];
        for await (const chunk of req) { record(chunk); chunks.push(chunk); }
        const requestBody = Buffer.concat(chunks);
        record(requestBody);
        const headers = { ...req.headers };
        delete headers['accept-encoding'];
        const upstream = request({ host: '127.0.0.1', port: 38079, path: req.url, method: req.method, headers }, response => {
          const responseChunks = [];
          try {
            record(Buffer.from(`${response.statusCode}\n${JSON.stringify(response.headers)}`));
            res.writeHead(response.statusCode, response.headers);
          } catch (error) { observerError = error; response.destroy(); res.destroy(); return; }
          response.on('data', chunk => {
            try { record(chunk); responseChunks.push(chunk); res.write(chunk); }
            catch (error) { observerError = error; response.destroy(); res.destroy(); }
          });
          response.on('end', () => {
            try {
              const responseBody = Buffer.concat(responseChunks);
              record(responseBody);
              exchanges.push({ path: req.url, method: req.method, status: response.statusCode, requestBody, responseBody });
              res.end();
            } catch (error) { observerError = error; res.destroy(); }
          });
          response.on('error', () => { observerError = new Error('coordinator response stream failed'); res.destroy(); });
        });
        upstream.on('error', () => { observerError = new Error('coordinator observer forwarding failed'); res.destroy(); });
        upstream.end(requestBody);
      } catch (error) {
        observerError = error;
        res.destroy();
      }
    });
    observer.listen(38080, '127.0.0.1');
    await once(observer, 'listening');
    browser = await chromium.launch({ executablePath: '/usr/bin/chromium', headless: true, args: ['--no-sandbox'] });
    const context = await browser.newContext({ acceptDownloads: true, viewport: { width: 1280, height: 800 } });
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);
    await context.route('**/*', route => {
      const url = new URL(route.request().url());
      if (url.origin !== origin) {
        observerError = new Error('browser attempted a non-fixture HTTP endpoint');
        return route.abort();
      }
      browserRequests.push(Buffer.from(`${route.request().method()} ${url}\n${JSON.stringify(route.request().headers())}\n${route.request().postData() || ''}`));
      return route.continue();
    });
    const page = await context.newPage();
    page.setDefaultTimeout(30_000);
    const cdp = await context.newCDPSession(page);
    await cdp.send('WebAuthn.enable');
    await cdp.send('WebAuthn.addVirtualAuthenticator', { options: {
      protocol: 'ctap2', transport: 'internal', hasResidentKey: true, hasUserVerification: true,
      isUserVerified: true, automaticPresenceSimulation: true,
    } });
    stage = 'real passkey registration and encrypted root setup';
    await page.goto(`${origin}/auth`);
    await page.locator('#create-passkey').click();
    await page.waitForURL('**/setup/node');
    await page.locator('#create-age').click();
    await page.locator('#copy-age').click();
    const ownerIdentity = remember(await page.evaluate(() => navigator.clipboard.readText()));
    assert.match(ownerIdentity, /^AGE-SECRET-KEY-PQ-1/);
    await page.locator('#age-backed-up').check();
    await page.locator('#create-root').click();
    await page.locator('#root-fingerprint').getByText(/Instance root fingerprint:/).waitFor();
    const fingerprint = (await page.locator('#root-fingerprint').innerText()).match(/[0-9a-f]{64}/i)?.[0];
    assert.ok(fingerprint);
    await page.locator('#copy-root').click();
    const rootBackup = await page.evaluate(() => navigator.clipboard.readText());
    assert.match(rootBackup, /^-----BEGIN AGE ENCRYPTED FILE-----/);

    stage = 'real metadata enrollment and browser-signed approval';
    const shared = join(root, 'shared');
    const data = join(root, 'data');
    await mkdir(shared);
    await mkdir(data);
    const metadata = await start('metadata', 38081, {
      RAWARR_SHARED_DATA_DIR: shared, RAWARR_METADATA_DIR: join(shared, 'in/metadata'),
      RAWARR_DATA_DIR: data, RAWARR_NODE_ID: nodeId, RAWARR_DEVICE_ID: nodeId,
      RAWARR_DEVICE_NAME: 'Disposable browser enrollment metadata fixture',
      RAWARR_NODE_TOKEN_FILE: join(data, 'node-token'), RAWARR_NODE_ENROLLMENT_ENABLED: 'true',
      RAWARR_COORDINATOR_URL: origin, RAWARR_COORDINATOR_LOCAL_HTTP: 'true',
      RAWARR_INSTANCE_ROOT_FINGERPRINT: fingerprint,
      RAWARR_WEBAUTHN_RP_ID: 'localhost', RAWARR_WEBAUTHN_ORIGIN: 'http://localhost:38081',
    });
    await page.goto(origin);
    await page.locator('[data-view="fossils"]').click();
    const pending = page.locator('#pending .pending-item').filter({ hasText: nodeId });
    await pending.waitFor();
    await pending.getByRole('button', { name: 'Approve', exact: true }).click();
    await page.locator('#approve-node-age-id').fill(ownerIdentity);
    await page.locator('#approve-node-root-backup').fill(rootBackup);
    await page.locator('#approve-node-root-fingerprint').fill(fingerprint);
    const approval = page.waitForResponse(response => response.url().endsWith('/api/workspace/approve') && response.request().method() === 'POST');
    await page.locator('#approve-node-form [type="submit"]').click();
    assert.equal((await approval).status(), 204);
    await page.locator('#approve-node-dialog').waitFor({ state: 'hidden' });
    await pending.waitFor({ state: 'detached' });
    await ready(metadata, 38081);
    remember((await readFile(join(data, 'node-age-identity'), 'utf8')).split('\n').find(line => line.startsWith('AGE-SECRET-KEY-PQ-1')));
    console.log('PASS real Chromium passkey registration, encrypted root creation, metadata enrollment and signed approval');
    if (process.argv.includes('--inject-failure')) throw new Error('injected failure after enrollment');

    stage = 'synthetic encrypted metadata preparation';
    const groupIdentity = remember(identity());
    const wrongIdentity = remember(identity());
    const recipient = publicKey(groupIdentity);
    const filenameKey = randomBytes(32);
    remember(filenameKey.toString('hex'));
    const upstreamCredential = remember(`fixture-upstream-${randomBytes(24).toString('hex')}`);
    const config = Buffer.from(JSON.stringify({ metadata_filename_key: filenameKey.toString('hex'), radarr_api_key: upstreamCredential }));
    await writeFile(join(shared, 'config.aged'), encrypt(recipient, config));
    await writeFile(join(shared, 'group-age-recipient'), recipient);
    await writeFile(join(shared, 'age-recipient'), recipient);
    await writeFile(join(shared, 'group-age-user-envelope.aged'), encrypt(publicKey(ownerIdentity), Buffer.from(groupIdentity)));
    for (const kind of ['text', 'artwork']) await mkdir(join(shared, 'in/metadata/movies', kind), { recursive: true });
    for (let index = 0; index < 3; index++) {
      const movie = { tmdbId: 870000 + index, schemaVersion: 1, title: remember(`private-movie-${randomBytes(18).toString('hex')}`) };
      const artwork = Buffer.concat([Buffer.from([0xff, 0xd8, 0xff]), Buffer.from(remember(`private-artwork-${randomBytes(18).toString('hex')}`)), Buffer.from([0xff, 0xd9])]);
      const filename = (kind, suffix) => createHmac('sha256', filenameKey)
        .update(`rawrarr-metadata-filename-v1\0${kind}\0tmdb\0${movie.tmdbId}`).digest('hex') + suffix;
      const textName = filename('text', '.json.aged');
      const artworkName = filename('artwork', '.aged');
      const textCiphertext = encrypt(recipient, Buffer.from(JSON.stringify(movie)));
      const artworkCiphertext = encrypt(recipient, Buffer.concat([Buffer.from('image/jpeg\n'), artwork]));
      await writeFile(join(shared, 'in/metadata/movies/text', textName), textCiphertext);
      await writeFile(join(shared, 'in/metadata/movies/artwork', artworkName), artworkCiphertext);
      records.push({ movie, artwork, textName, artworkName, textCiphertext, artworkCiphertext });
    }
    // Drop the owner session and prove a new WebAuthn assertion, not an injected cookie, signs it back in.
    stage = 'real passkey sign-in';
    await context.clearCookies();
    await page.goto(`${origin}/auth`);
    await page.locator('#sign-in').click();
    await page.waitForURL(origin + '/');
    let downloadCount = 0;
    page.on('download', () => { downloadCount++; });
    for (const viewport of [{ width: 1280, height: 800 }, { width: 375, height: 812 }]) {
      await page.setViewportSize(viewport);
      stage = `wrong-key control at ${viewport.width}`;
      await page.goto(`${origin}/?node=${nodeId}#settings`);
      await page.locator('[data-view="settings"]').click();
      if (await page.locator('#remove-age-id').isVisible()) await page.locator('#remove-age-id').click();
      await layout(page, '#export-metadata');
      await page.locator('#export-metadata').click();
      await page.locator('#metadata-export-identity').fill(wrongIdentity);
      await layout(page, '#metadata-export-start');
      const before = downloadCount;
      await page.locator('#metadata-export-start').click();
      await page.waitForFunction(() => /Could not decrypt/i.test(document.getElementById('metadata-export-status').textContent));
      assert.equal(downloadCount, before, 'wrong valid identity must not download');
      await page.locator('#remove-age-id').click();

      stage = `real browser export at ${viewport.width}`;
      await page.locator('#export-metadata').click();
      await page.locator('#metadata-export-identity').fill(ownerIdentity);
      const downloadPromise = page.waitForEvent('download');
      await page.locator('#metadata-export-start').click();
      const download = await downloadPromise;
      assert.equal(download.suggestedFilename(), 'rawrarr-metadata-export.zip');
      const zipPath = join(root, `export-${viewport.width}.zip`);
      await download.saveAs(zipPath);
      const archive = unzipSync(await readFile(zipPath));
      const manifest = JSON.parse(Buffer.from(archive['manifest.json']).toString('utf8'));
      assert.equal(manifest.movies.length, records.length);
      assert.equal(Object.keys(archive).length, 1 + records.length * 2);
      for (const { movie, artwork, textName, artworkName } of records) {
        assert.deepEqual(JSON.parse(Buffer.from(archive[`movies/text/${movie.tmdbId}.json`]).toString('utf8')), movie);
        assert.deepEqual(Buffer.from(archive[`movies/artwork/${movie.tmdbId}.jpg`]), artwork);
        assert.deepEqual(manifest.movies.find(entry => entry.tmdbId === movie.tmdbId), {
          tmdbId: movie.tmdbId, title: movie.title, textPath: `movies/text/${movie.tmdbId}.json`,
          textCiphertextPath: `movies/text/${textName}`, artworkPath: `movies/artwork/${movie.tmdbId}.jpg`,
          artworkCiphertextPath: `movies/artwork/${artworkName}`,
        });
      }
      await page.waitForFunction(() => document.getElementById('metadata-export-status').textContent.startsWith('Downloaded 3 '));
      console.log(`PASS wrong valid key rejected without download; real relay ciphertext decrypted to exact ZIP at ${viewport.width}x${viewport.height}`);
    }

    stage = 'unauthorized browser controls';
    const anonymous = await browser.newContext();
    const denied = await anonymous.newPage();
    const deniedPaths = ['mini', 'config', 'recipient', 'node-recipient', 'group-user-envelope',
      `ciphertext/text/${records[0].textName}`, `ciphertext/artwork/${records[0].artworkName}`];
    for (const [base, prefix] of [[origin, relay], ['http://localhost:38081', '/api/metadata/export']]) {
      await denied.goto(base + '/health');
      const statuses = await denied.evaluate(async ({ prefix, paths }) => {
        const results = [];
        for (const path of paths) {
          results.push((await fetch(`${prefix}/${path}`, { credentials: 'omit' })).status);
        }
        return results;
      }, { prefix, paths: deniedPaths });
      assert.deepEqual(statuses, deniedPaths.map(() => 401));
    }
    for (const token of [null, 'invalid-fixture-session']) {
      for (const base of [origin + relay, 'http://localhost:38081/api/metadata/export']) {
        for (const path of deniedPaths) {
          const response = await anonymous.request.get(`${base}/${path}`, {
            headers: token ? { Cookie: `rawrarr_session=${token}` } : {},
          });
          assert.equal(response.status(), 401);
          assertPrivate(await response.body(), markers, 'unauthorized response');
        }
      }
    }
    await anonymous.close();
    stage = 'browser request evidence';
    const requestsText = Buffer.concat(browserRequests).toString('utf8');
    assert.ok(requestsText.includes(`${relay}/ciphertext/text/`));
    assert.ok(requestsText.includes(`${relay}/ciphertext/artwork/`));
    assert.ok(requestsText.includes('/api/auth/register/finish'));
    assert.ok(requestsText.includes('/api/auth/login/finish'));
    assert.ok(requestsText.includes('/api/workspace/approve'));
    stage = 'real relay ciphertext evidence';
    for (const { textName, artworkName, textCiphertext, artworkCiphertext } of records) {
      for (const [path, ciphertext] of [[`text/${textName}`, textCiphertext], [`artwork/${artworkName}`, artworkCiphertext]]) {
        const transfers = exchanges.filter(exchange => exchange.path === `${relay}/ciphertext/${path}` && exchange.status === 200);
        assert.ok(transfers.length >= 2, 'both viewport flows must traverse the real coordinator relay');
        for (const transfer of transfers) assert.deepEqual(transfer.responseBody, ciphertext);
      }
    }
    stage = 'service shutdown';
    await context.close();
    await browser.close();
    browser = null;
    await stopServices();
    stage = 'coordinator boundary audit';
    assertPrivate(Buffer.concat(observed), markers, 'coordinator HTTP ingress/egress');
    assertPrivate(Buffer.concat(browserRequests), markers, 'browser requests');
    for (const { logPath } of children) assertPrivate(await readFile(logPath), markers, 'service logs');
    if (observerError) throw observerError;
    await writeFile(auditPath, JSON.stringify({ markers }), { mode: 0o600 });
    console.log('PASS unauthenticated/invalid-session relay and node requests denied; observed coordinator traffic and service logs contain no private fixture markers');
  } catch (error) {
    // Avoid raw Playwright call logs/DOM values that may contain private form contents.
    if (String(error.message).startsWith('injected failure')) throw error;
    const frames = String(error.stack || '').split('\n').filter(line => /^\s+at /.test(line)).join('\n');
    throw new Error(`browser enrollment/export failed during ${stage} (${error.name}); private diagnostics suppressed\n${frames}`);
  } finally {
    await browser?.close();
    await stopServices();
    if (observer) {
      observer.closeAllConnections();
      await new Promise(resolve => observer.close(resolve));
    }
    await rm(root, { recursive: true, force: true });
  }
}
