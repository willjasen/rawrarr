import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const compose = await readFile(new URL('../../deploy/do3/docker-compose.internal-coordinator.yml', import.meta.url), 'utf8');

function serviceBlock(service) {
  const lines = compose.split(/\r?\n/);
  const start = lines.findIndex(line => line === `  ${service}:`);
  assert.notEqual(start, -1, `missing Compose service ${service}`);
  let end = start + 1;
  while (end < lines.length && !/^(?:  \S[^:]*:|\S)/.test(lines[end])) end++;
  return lines.slice(start, end).join('\n');
}

test('do3 internal coordinator has separate PostgreSQL state and no public database port', () => {
  const postgres = serviceBlock('postgres');
  assert.match(postgres, /container_name: rawrarr-internal-do3-postgres/);
  assert.match(postgres, /rawrarr-internal-do3-postgres-data/);
  assert.doesNotMatch(postgres, /^\s+ports\s*:/m);
});

test('do3 internal ingress uses the moved rawrarr Tailscale identity and loopback diagnostics', () => {
  const tailscale = serviceBlock('tailscale');
  assert.match(tailscale, /container_name: rawrarr-internal-do3-tailscale/);
  assert.match(tailscale, /hostname: rawrarr/);
  assert.match(tailscale, /rawrarr-internal-do3-tailscale-state/);
  assert.match(tailscale, /127\.0\.0\.1:7299:8080/);
  assert.match(tailscale, /TS_USERSPACE: "true"/);
});

test('internal roarlay retains the passkey origin, internal billing audience, and Tailscale namespace', () => {
  const roarlay = serviceBlock('roarlay');
  assert.match(roarlay, /container_name: rawrarr-internal-do3-roarlay/);
  assert.match(roarlay, /pull_policy: never/);
  assert.match(roarlay, /network_mode: service:tailscale/);
  assert.match(roarlay, /RAWARR_BILLING_AUDIENCE: internal/);
  assert.match(roarlay, /RAWARR_UI_GITHUB_TOKEN_FILE: \/run\/secrets\/rawrarr_ui_github_token/);
  assert.match(roarlay, /- rawrarr_ui_github_token/);
  assert.match(roarlay, /RAWARR_COORDINATOR_WEB_RP_ID: risk-mermaid\.ts\.net/);
  assert.match(roarlay, /RAWARR_COORDINATOR_WEB_ORIGIN: https:\/\/rawrarr\.risk-mermaid\.ts\.net/);
  assert.doesNotMatch(roarlay, /^\s+ports\s*:/m);
});

test('internal project keeps its Docker networks and app data separate from public and subscriber paths', () => {
  assert.match(compose, /name: rawrarr-internal-do3-control\n    internal: true/);
  assert.match(compose, /name: rawrarr-internal-do3-egress/);
  assert.match(compose, /name: rawrarr-internal-do3-ui/);
  assert.doesNotMatch(compose, /\/shared|\/media|rawrarr-roarlay-prod-control/);
  for (const path of ['approved-nodes.json', 'openai-api-key', 'postgres-password']) {
    assert.ok(compose.includes(`/opt/rawrarr-internal-do3/secrets/${path}`), `missing protected host secret path ${path}`);
  }
  assert.match(compose, /RAWARR_UI_GITHUB_TOKEN_PATH:-\/root\/gh-token/);
});
