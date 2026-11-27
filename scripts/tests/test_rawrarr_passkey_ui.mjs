import assert from 'node:assert/strict';
import test from 'node:test';
import { decodeOptions } from '../../rust-arr/src/passkey-client.mjs';
import { readJsonResponse } from '../../rust-arr/src/auth-response.mjs';
import { readFile } from 'node:fs/promises';

const authPage = await readFile(new URL('../../rust-arr/src/auth.html', import.meta.url), 'utf8');
const authSource = await readFile(new URL('../../rust-arr/src/auth.rs', import.meta.url), 'utf8');

const encoded = value => Buffer.from(value).toString('base64url');

test('handles empty and non-JSON sign-in error responses without a JSON parse exception', async () => {
  await assert.rejects(
    readJsonResponse(new Response('', { status: 404 }), 'Passkey sign-in failed'),
    { message: 'This sign-in method is unavailable in this environment.' },
  );
  await assert.rejects(
    readJsonResponse(new Response('upstream unavailable', { status: 503 }), 'Passkey sign-in failed'),
    { message: 'Passkey sign-in failed (HTTP 503)' },
  );
  await assert.rejects(
    readJsonResponse(new Response('{"error":"Wallet service is unavailable"}', { status: 503 }), 'Passkey sign-in failed'),
    { message: 'Wallet service is unavailable' },
  );
});

test('decodes the nested WebAuthn publicKey response shape', () => {
  const options = decodeOptions({
    publicKey: {
      challenge: encoded('challenge'),
      user: { id: encoded('user-id'), name: 'owner' },
      allowCredentials: [{ id: encoded('credential-id'), type: 'public-key' }],
      rpId: 'localhost',
    },
  });

  assert.deepEqual([...options.challenge], [...Buffer.from('challenge')]);
  assert.deepEqual([...options.user.id], [...Buffer.from('user-id')]);
  assert.deepEqual([...options.allowCredentials[0].id], [...Buffer.from('credential-id')]);
  assert.equal(options.rpId, 'localhost');
});

test('reports incomplete server options without a replace TypeError', () => {
  assert.throws(
    () => decodeOptions({}),
    { message: 'The server returned incomplete passkey options. Reload the page and try again.' },
  );
  assert.throws(
    () => decodeOptions({ publicKey: {} }),
    { message: 'The server returned incomplete passkey options. Reload the page and try again.' },
  );
});

test('offers passkey sign-in and account creation without username or password sign-in', () => {
  assert.match(authPage, /id="sign-in" type="button">Sign in with passkey/);
  assert.match(authPage, /id="create-passkey" type="button">Create Passkey/);
  assert.match(authPage, /<section id="register-panel">/);
  assert.ok(authPage.indexOf('id="sign-in"') < authPage.indexOf('id="create-passkey"'));
  assert.doesNotMatch(authPage, /id="username"|id="new-username"/);
  assert.doesNotMatch(authPage, /id="display-name"|autocomplete="username"/);
  assert.match(authPage, /document\.getElementById\('login-panel'\)\.hidden = !status\.userExists/);
  assert.match(authPage, /document\.getElementById\('register-panel'\)\.hidden = true/);
  assert.match(authPage, /post\('\/api\/auth\/login\/start', \{\}\)/);
  assert.match(authPage, /post\('\/api\/auth\/register\/start', \{\}\)/);
  assert.match(authPage, /id="sign-in-ens" type="button">Sign in with ENS wallet/);
});

test('allows additional passkey accounts and authenticates against all registered passkeys', () => {
  assert.doesNotMatch(authSource, /First-user setup is already complete/);
  assert.match(authSource, /FROM rawrarr_users u JOIN rawrarr_passkeys p ON p\.user_id = u\.id ORDER BY u\.created_at, p\.created_at/);
  assert.match(authSource, /result\.cred_id\(\)\.as_ref\(\)/);
  assert.match(authSource, /if !exists \{\s*sqlx::query\("UPDATE rawrarr_nodes SET user_id = \$1 WHERE user_id IS NULL"\)/);
  assert.match(authSource, /struct AuthenticationStart \{\}/);
});

 test('both ENS wallet linking requests declare their JSON bodies', async () => {
  const workspace = await readFile(new URL('../../rust-arr/src/coordinator-workspace.mjs', import.meta.url), 'utf8');
  for (const phase of ['start', 'finish']) {
    const line = workspace.split('\n').find(line => line.includes(`/api/auth/eth/link/${phase}`));
    assert.ok(line.includes("'Content-Type': 'application/json'"), `${phase} must declare JSON to avoid HTTP 415`);
  }
});
