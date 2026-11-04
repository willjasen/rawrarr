import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const compose = await readFile(new URL('../../deploy/do3/docker-compose.production-container-names.yml', import.meta.url), 'utf8');

const services = [
  'caddy',
  'cloudflared',
  'crowdsec',
  'headscale',
  'postgres',
  'roarlay',
  'stripe-service',
  'subscriber-provisioner',
  'swarmsail',
  'tailscale',
  'umami',
  'umami-postgres',
];

test('production services use the rawrarr-prod container prefix', () => {
  for (const service of services) {
    const block = compose.match(new RegExp(`^  ${service}:\\n((?:    .*\\n|\\n)*)`, 'm'));
    assert.ok(block, `missing production service ${service}`);
    assert.match(block[1], new RegExp(`^    container_name: rawrarr-prod-${service}$`, 'm'));
  }
});
