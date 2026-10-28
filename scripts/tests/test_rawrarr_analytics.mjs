import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [coordinator, analyticsCompose, publicCompose, miniCompose, do3Compose] = await Promise.all([
  read('rust-arr/src/coordinator.rs'),
  read('docker-compose.analytics.yml'),
  read('docker-compose.coordinator.yml'),
  read('docker-compose.internal-mini.yml'),
  read('deploy/do3/docker-compose.internal-coordinator.yml'),
]);

test('coordinator analytics are opt-in and only emit a fixed page label', () => {
  assert.match(coordinator, /RAWARR_ANALYTICS_SITE_ID/);
  assert.match(coordinator, /RAWARR_ANALYTICS_UPSTREAM/);
  assert.match(coordinator, /url\.host_str\(\) == Some\("umami"\)/);
  assert.match(coordinator, /url\.port\(\) == Some\(3000\)/);
  assert.match(coordinator, /Method::GET, "script\.js"/);
  assert.match(coordinator, /Method::POST, "api\/send"/);
  assert.match(coordinator, /get\("true-client-ip"\)[\s\S]*?get\("x-forwarded-for"\)/);
  assert.match(coordinator, /outbound\.header\("x-forwarded-for", value\)/);
  assert.doesNotMatch(coordinator, /header::X_FORWARDED_FOR/);
  assert.match(coordinator, /data-auto-track="false"/);
  assert.match(coordinator, /website:'\{site_id\}',url:'\/coordinator',title:'Rawrarr coordinator'/);
  assert.doesNotMatch(coordinator, /umami\.track\(\)/);
});

test('analytics has an isolated pinned Umami and PostgreSQL deployment', () => {
  assert.match(analyticsCompose, /image: ghcr\.io\/umami-software\/umami:3\.4\.1/);
  assert.match(analyticsCompose, /127\.0\.0\.1:\$\{RAWARR_ANALYTICS_PORT:-7300\}:3000/);
  assert.match(analyticsCompose, /umami-postgres-data:/);
  assert.match(analyticsCompose, /external: true/);
  assert.match(analyticsCompose, /name: \$\{RAWARR_ANALYTICS_NETWORK/);
  assert.doesNotMatch(analyticsCompose, /rawrarr-postgres-data|rawarr-internal/);
});

test('analytics overlay configures both coordinator and Umami services without telemetry', () => {
  assert.match(analyticsCompose, /RAWARR_ANALYTICS_UPSTREAM: http:\/\/umami:3000/);
  assert.match(analyticsCompose, /RAWARR_ANALYTICS_SITE_ID: \$\{RAWARR_ANALYTICS_SITE_ID:-\}/);
  assert.match(analyticsCompose, /DISABLE_TELEMETRY: "1"/);
  assert.match(do3Compose, /RAWARR_ROLE: coordinator/);
  assert.match(publicCompose, /RAWARR_ROLE: coordinator/);
  assert.match(miniCompose, /RAWARR_ROLE: coordinator/);
});
