import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [base, metadata, storage, orbstackEnvironment] = await Promise.all([
  read('docker-compose.yml'),
  read('docker-compose.metadata.yml'),
  read('docker-compose.storage.yml'),
  read('deploy/testing/compose.yml'),
]);

function serviceBlock(compose, service) {
  const lines = compose.split(/\r?\n/);
  const start = lines.findIndex(line => line === `  ${service}:`);
  assert.notEqual(start, -1, `missing Compose service ${service}`);
  let end = start + 1;
  while (end < lines.length && !/^(?:  \S[^:]*:|\S)/.test(lines[end])) end++;
  return lines.slice(start, end).join('\n');
}

test('metadata and storage have no host-published application ports', () => {
  for (const [compose, services] of [
    [base, ['fossil']],
    [metadata, ['fossil', 'artwork-cache']],
    [storage, ['boneyard']],
    [orbstackEnvironment, ['metadata-a', 'metadata-b', 'tailscale-a', 'tailscale-b']],
  ]) {
    for (const service of services) {
      assert.doesNotMatch(serviceBlock(compose, service), /^\s+ports\s*:/m, `${service} must not publish container ports to the host`);
    }
  }
});

test('isolated test subscribers bind loopback in their own Tailscale namespaces', () => {
  for (const [service, sidecar] of [['metadata-a', 'tailscale-a'], ['metadata-b', 'tailscale-b']]) {
    const block = serviceBlock(orbstackEnvironment, service);
    assert.match(block, new RegExp(`network_mode: service:${sidecar}`));
    assert.match(block, /RUST_ARR_LISTEN_ADDR: 127\.0\.0\.1:8080/);
  }
});

test('standalone metadata Compose identifies the service and device as fossil', () => {
  assert.match(metadata, /services:[\s\S]*?\n  fossil:/);
  assert.match(metadata, /RAWARR_DEVICE_NAME: \$\{RAWARR_DEVICE_NAME:-Fossil node\}/);
});

test('metadata artwork cache uses Valkey with bounded disposable storage and remains opt-in', () => {
  for (const compose of [metadata]) {
    assert.match(compose, /image: valkey\/valkey:8-alpine/);
    assert.match(compose, /valkey-server/);
    assert.match(compose, /--save", "", "--appendonly", "no"/);
    assert.match(compose, /--maxmemory", "2gb", "--maxmemory-policy", "allkeys-lru"/);
    assert.match(compose, /valkey-cli", "ping/);
    assert.match(compose, /mem_limit: 2560m/);
    assert.doesNotMatch(compose, /image: redis:/);
  }
  assert.match(metadata, /RAWARR_ARTWORK_CACHE_ENABLED: \$\{RAWARR_ARTWORK_CACHE_ENABLED:-false\}/);
  assert.match(metadata, /RAWARR_ARTWORK_CACHE_URL: redis:\/\/artwork-cache:6379\//);
});

test('standalone storage uses loopback in an externally managed Tailscale namespace', () => {
  assert.match(storage, /network_mode: "container:\$\{RAWARR_TAILSCALE_CONTAINER:-rawrarr-mini-tailscale\}"/);
  assert.match(storage, /RUST_ARR_LISTEN_ADDR: 127\.0\.0\.1:8080/);
  assert.match(storage, /RUST_ARR_DIRECT_LISTEN_ADDR: 127\.0\.0\.1:8083/);
  assert.match(storage, /RAWARR_DIRECT_ALLOWED_ORIGIN: https:\/\/rawrarr\.risk-mermaid\.ts\.net/);
});
