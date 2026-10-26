import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [coordinator, main, nodeAuth, workspace, script, routes, compose, storageCompose] = await Promise.all([
  read('rust-arr/src/coordinator.rs'),
  read('rust-arr/src/main.rs'),
  read('rust-arr/src/node_auth.rs'),
  read('rust-arr/src/coordinator_workspace.html'),
  read('rust-arr/src/coordinator-workspace.mjs'),
  read('rust-arr/src/coordinator-device-routes.mjs'),
  read('docker-compose.tailscale-macbook.yml'),
  read('docker-compose.storage.yml'),
]);

test('coordinator advertises configured coordinator or tailnet direct routes to the signed-in owner', () => {
  assert.match(coordinator, /RAWARR_NODE_DIRECT_TARGETS/);
  assert.match(coordinator, /fn validate_direct_target\(origin: &str, target: &str\)/);
  assert.match(coordinator, /host\.ends_with\("\.risk-mermaid\.ts\.net"\)/);
  assert.match(coordinator, /host == "rawrarr\.risk-mermaid\.ts\.net"[\s\S]*host\.starts_with\("rawrarr-"\)/);
  assert.match(coordinator, /state\.direct_targets\.get\(&node_id\)\.map\(url::Url::to_string\)/);
  assert.match(coordinator, /api\/workspace\/direct-session/);
  assert.match(coordinator, /headers\.get\(header::ORIGIN\).*state\.web_origin/);
  assert.match(coordinator, /SELECT role FROM rawrarr_nodes WHERE id=\$1 AND user_id=\$2/);
  assert.match(coordinator, /DIRECT_SESSION_SECONDS: i64 = 300/);
  assert.match(coordinator, /Path=\{\}; HttpOnly; Secure; SameSite=Strict/);
  assert.match(coordinator, /DELETE FROM rawrarr_sessions WHERE user_id=\$1 AND node_id <> 'coordinator'/);
  assert.match(coordinator, /rawrarr_direct_session=; Path=\{\}; HttpOnly; Secure; SameSite=Strict; Max-Age=0/);
});

test('metadata and storage direct listeners expose health and session-protected routes only', () => {
  assert.match(main, /RUST_ARR_DIRECT_LISTEN_ADDR/);
  assert.match(main, /direct API listener is available only for metadata or storage nodes/);
  assert.match(main, /api\/direct\/health/);
  assert.match(main, /api\/direct\/session/);
  assert.match(main, /RAWARR_DIRECT_ALLOWED_ORIGIN/);
  assert.match(main, /access-control-allow-origin/);
  assert.match(main, /Access-Control-Allow-Headers|access-control-allow-headers/);
  assert.match(main, /direct_protected[\s\S]*?api\/metadata\/export\/mini[\s\S]*?api\/metadata\/session\/library[\s\S]*?api\/metadata\/export\/config[\s\S]*?api\/metadata\/export\/ciphertext/);
  const protectedRoutes = main.match(/let direct_protected = Router::new\(\)([\s\S]*?)\.route_layer/)?.[1] || '';
  assert.match(protectedRoutes, /\/api\/settings\/age/);
  assert.match(protectedRoutes, /\/api\/settings\/radarr/);
  assert.match(protectedRoutes, /\/api\/settings\/sonarr/);
  assert.match(protectedRoutes, /\/api\/radarr\/live-library/);
  assert.doesNotMatch(protectedRoutes, /\/api\/auth\/(?:login|logout|register)/);
  assert.match(nodeAuth, /direct_session_token\(headers\)\.or_else\(\|\| crate::auth::cookie_token\(headers\)\)/);
});

test('workspace automatically selects direct Tailscale access, visibly reports it, and falls back to coordinator relay', () => {
  assert.doesNotMatch(workspace, /id="tv-route-status" class="route-status"/);
  assert.match(script, /state\.className = `node-state \$\{node\.online \? 'online' : 'offline'\} \$\{routeState\}`/);
  assert.match(script, /direct: 'Direct',[\s\S]*relay: 'Relay',[\s\S]*offline: node\.online \? 'Route unavailable' : 'Not checking in'/);
  assert.match(script, /nodeStateLabel = node\.online \? `Online · \$\{routeLabel\}` : routeLabel/);
  assert.match(script, /aria-label', route\?\.status === 'direct' && latencyLabel/);
  assert.match(routes, /export async function checkDeviceRoutes\(devices/);
  assert.match(routes, /status, directRoute/);
  assert.match(routes, /api\/direct\/health/);
  assert.match(routes, /credentials: 'omit'/);
  assert.match(routes, /mode: 'cors'/);
  assert.match(coordinator, /assets\/coordinator-device-routes\.mjs/);
  assert.match(script, /api\/workspace\/direct-session/);
  assert.match(script, /api\/direct\/session/);
  assert.match(script, /credentials: 'include'/);
  assert.match(script, /if \(view === 'series' && updateHash\)[\s\S]*?chooseNodeRoute\(selectedNodeDetails, true\)/);
  assert.match(script, /if \(deviceRoute\?\.status === 'direct'\) return chooseDirectRoute\(node, deviceRoute\.directRoute\)/);
  assert.match(script, /const relayRoute = \{ nodeId: node\.nodeId, base: `\/nodes\/\$\{encodeURIComponent\(node\.nodeId\)\}`, direct: false \}/);
  assert.match(script, /Direct Tailscale route active/);
  assert.match(compose, /RAWARR_NODE_DIRECT_TARGETS/);
  assert.match(compose, /RUST_ARR_DIRECT_LISTEN_ADDR: 127\.0\.0\.1:8082/);
  assert.match(compose, /mac-mini-storage.*rawrarr-mini\.risk-mermaid\.ts\.net/);
  assert.match(storageCompose, /RUST_ARR_DIRECT_LISTEN_ADDR: 127\.0\.0\.1:8083/);
  assert.match(storageCompose, /RAWARR_DIRECT_ALLOWED_ORIGIN: https:\/\/rawrarr\.risk-mermaid\.ts\.net/);
});
