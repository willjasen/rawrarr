import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [metadataUi, ageSetup, coordinatorPage, coordinatorScript, coordinator, migration, compose, publicCompose] = await Promise.all([
  read('rust-arr/src/ui.html'),
  read('rust-arr/src/age_setup.html'),
  read('rust-arr/src/coordinator_passkey.html'),
  read('rust-arr/src/coordinator-passkey.mjs'),
  read('rust-arr/src/coordinator.rs'),
  read('rust-arr/migrations/0005_coordinator_origin_passkey.sql'),
  read('docker-compose.yml'),
  read('docker-compose.coordinator.yml'),
]);

test('signed-in metadata UI starts a coordinator-origin passkey handoff', () => {
  assert.match(metadataUi, /id="add-coordinator-passkey"[^>]*hidden[^>]*>Add passkey for roarlay/);
  assert.match(metadataUi, /window\.location\.origin === 'http:\/\/localhost:7297'/);
  assert.match(metadataUi, /\/api\/auth\/passkeys\/coordinator\/link/);
  assert.match(metadataUi, /window\.location\.assign\(result\.url\)/);
  assert.match(ageSetup, /id="coordinator-passkey-section"[^>]*hidden/);
  assert.match(ageSetup, /window\.location\.origin !== 'http:\/\/localhost:7297'/);
  assert.match(ageSetup, /\/api\/auth\/passkeys\/coordinator\/link/);
  assert.match(coordinator, /authenticated_user_id\(&auth, &headers\)/);
  assert.match(coordinator, /INSERT INTO rawrarr_passkey_link_tokens\(token_sha256, user_id, source_node_id, expires_at\)/);
  assert.match(coordinator, /now\(\) \+ interval '5 minutes'/);
});

test('one-time link is removed from history and ceremony completes on coordinator origin', () => {
  assert.match(coordinatorPage, /roarlay-site credential/);
  assert.match(coordinatorPage, /coordinator-passkey\.mjs/);
  assert.match(coordinatorPage, /If this message stays here, reload the page/);
  assert.match(coordinatorPage, /id="create-passkey"[^>]*>Create roarlay passkey/);
  assert.match(coordinatorScript, /passkey-client\.mjs\?v=coordinator-passkey-2/);
  assert.match(coordinatorScript, /location\.hash\.slice\(1\)/);
  assert.match(coordinatorScript, /history\.replaceState\(null, '', '\/passkeys\/add'\)/);
  assert.match(coordinatorScript, /Create roarlay passkey, then follow your browser/);
  assert.match(coordinatorScript, /navigator\.credentials\.create/);
  assert.match(coordinatorScript, /createButton\.addEventListener\('click', addPasskey\)/);
  assert.match(coordinatorScript, /\/api\/passkeys\/coordinator\/register\/start/);
  assert.match(coordinatorScript, /\/api\/passkeys\/coordinator\/register\/finish/);
  assert.match(coordinator, /finish_passkey_registration\(&request\.credential, &registration\)/);
});

test('new coordinator credential is inserted for the handoff account and Tailscale RP is configured', () => {
  assert.match(coordinator, /INSERT INTO rawrarr_passkeys\(credential_id,user_id,passkey\)/);
  assert.match(coordinator, /SELECT l\.user_id,u\.username,u\.display_name/);
  assert.match(migration, /flow IN \('registration', 'authentication', 'add_passkey'\)/);
  assert.match(migration, /link_token_sha256/);
  assert.match(compose, /RAWARR_COORDINATOR_WEB_RP_ID: \$\{RAWARR_COORDINATOR_WEB_RP_ID:-rawrarr\.risk-mermaid\.ts\.net\}/);
  assert.match(compose, /RAWARR_COORDINATOR_WEB_ORIGIN: \$\{RAWARR_COORDINATOR_WEB_ORIGIN:-https:\/\/rawrarr\.risk-mermaid\.ts\.net\}/);
});

test('public Roarlay passkeys use the rawrarr.net RP ID at the app subdomain', () => {
  assert.match(publicCompose, /RAWARR_COORDINATOR_WEB_RP_ID: \$\{RAWARR_COORDINATOR_WEB_RP_ID:-rawrarr\.net\}/);
  assert.match(publicCompose, /RAWARR_COORDINATOR_WEB_ORIGIN: \$\{RAWARR_COORDINATOR_WEB_ORIGIN:-https:\/\/roarlay\.rawrarr\.net\}/);
});
