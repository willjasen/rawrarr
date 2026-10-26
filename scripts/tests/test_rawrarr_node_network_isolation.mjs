import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [base, metadata, storage, macbookTailscale, miniRelays, proxyConfig] = await Promise.all([
  read('docker-compose.yml'),
  read('docker-compose.metadata.yml'),
  read('docker-compose.storage.yml'),
  read('docker-compose.tailscale-macbook.yml'),
  read('docker-compose.mini-relays.yml'),
  read('storage-proxy.Caddyfile'),
]);

test('metadata and storage have no host-published application ports', () => {
  for (const compose of [base, metadata, storage]) {
    assert.doesNotMatch(compose, /127\.0\.0\.1:\$?\{?RAWARR_HOST_PORT[^\n]*:8080/);
    assert.doesNotMatch(compose, /127\.0\.0\.1:7297:8080/);
  }
});

test('MacBook metadata is loopback-only in the coordinator Tailscale namespace', () => {
  assert.match(macbookTailscale, /fossil:[\s\S]*?network_mode: "service:tailscale"/);
  assert.match(macbookTailscale, /RUST_ARR_LISTEN_ADDR: 127\.0\.0\.1:8081/);
  assert.match(macbookTailscale, /RAWARR_NODE_RELAY_TARGETS: .*127\.0\.0\.1:8081/);
  assert.match(macbookTailscale, /127\.0\.0\.1:8081\/health/);
});

test('standalone metadata Compose identifies the service and device as fossil', () => {
  assert.match(metadata, /services:[\s\S]*?\n  fossil:/);
  assert.match(metadata, /RAWARR_DEVICE_NAME: \$\{RAWARR_DEVICE_NAME:-Fossil node\}/);
});

test('metadata artwork cache uses Valkey with bounded disposable storage and remains opt-in', () => {
  for (const compose of [metadata, macbookTailscale]) {
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
  assert.match(macbookTailscale, /RAWARR_ARTWORK_CACHE_ENABLED: \$\{RAWARR_ARTWORK_CACHE_ENABLED:-false\}/);
  assert.match(macbookTailscale, /RAWARR_ARTWORK_CACHE_URL: redis:\/\/127\.0\.0\.1:6379\//);
  assert.match(macbookTailscale, /artwork-cache:[\s\S]*?network_mode: "service:tailscale"/);
});

test('Mini storage and Serve proxy use only loopback in the Tailscale namespace', () => {
  assert.match(storage, /network_mode: "container:\$\{RAWARR_TAILSCALE_CONTAINER:-rawrarr-mini-tailscale\}"/);
  assert.match(storage, /RUST_ARR_LISTEN_ADDR: 127\.0\.0\.1:8080/);
  assert.match(storage, /RUST_ARR_DIRECT_LISTEN_ADDR: 127\.0\.0\.1:8083/);
  assert.match(storage, /RAWARR_DIRECT_ALLOWED_ORIGIN: https:\/\/rawrarr\.risk-mermaid\.ts\.net/);
  assert.match(miniRelays, /storage-proxy\.Caddyfile/);
  assert.match(proxyConfig, /bind 127\.0\.0\.1/);
  assert.match(proxyConfig, /reverse_proxy 127\.0\.0\.1:8080/);
});
