import assert from 'node:assert/strict';
import test from 'node:test';
import { checkDeviceRoutes } from '../../rust-arr/src/coordinator-device-routes.mjs';

test('checks all devices and returns direct, relay, or offline route status', async () => {
  let calls = 0;
  const devices = [
    { nodeId: 'direct-device', online: true, relayAvailable: true, directUrl: 'https://direct.test/' },
    { nodeId: 'relay-device', online: true, relayAvailable: true, directUrl: 'https://relay.test/' },
    { nodeId: 'offline-device', online: false, relayAvailable: false, directUrl: 'https://offline.test/' },
  ];
  const routes = await checkDeviceRoutes(devices, {
    fetchImpl: async url => {
      calls++;
      return { ok: url.startsWith('https://direct.test/') };
    },
  });

  assert.equal(routes.get('direct-device').status, 'direct');
  assert.equal(routes.get('direct-device').directRoute.base, 'https://direct.test');
  assert.equal(routes.get('relay-device').status, 'relay');
  assert.equal(routes.get('offline-device').status, 'offline');
  assert.equal(calls, 2, 'offline devices do not need a network probe');
});

test('includes metadata-node Redis reachability in measured direct-route latency', async () => {
  const device = { nodeId: 'redis-metadata', online: true, relayAvailable: true, directUrl: 'https://redis.test/' };
  const routes = await checkDeviceRoutes([device], {
    fetchImpl: async () => ({
      ok: true,
      json: async () => ({ artworkCacheConfigured: true, artworkCacheReachable: true }),
    }),
  });
  const route = routes.get(device.nodeId).directRoute;
  assert.equal(route.artworkCacheConfigured, true);
  assert.equal(route.artworkCacheReachable, true);
  assert.ok(Number.isFinite(route.latencyMs));
});

test('reuses direct probe results across callers and can explicitly refresh them', async () => {
  let calls = 0;
  const device = { nodeId: 'cached-direct-device', online: true, relayAvailable: true, directUrl: 'https://cached-direct.test' };
  const options = { fetchImpl: async () => { calls++; return { ok: true }; } };

  await checkDeviceRoutes([device], options);
  await checkDeviceRoutes([device], options);
  assert.equal(calls, 1);

  await checkDeviceRoutes([device], { ...options, refresh: true });
  assert.equal(calls, 2);
});
