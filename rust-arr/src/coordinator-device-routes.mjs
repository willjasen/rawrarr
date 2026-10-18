const probeCache = new Map();
const probeCacheMilliseconds = 15_000;

/**
 * Check every node's usable browser route.
 *
 * The returned map is keyed by nodeId. Each value has status `direct`,
 * `relay`, or `offline`, and includes a directRoute when the browser can reach
 * the node's direct health endpoint. Pages can share this result for labels
 * and requests instead of maintaining independent route checks.
 */
export async function checkDeviceRoutes(devices, {
  fetchImpl = fetch,
  now = Date.now,
  refresh = false,
} = {}) {
  const results = await Promise.all(devices.map(async device => {
    if (!device.online) {
      return [device.nodeId, { nodeId: device.nodeId, status: 'offline', directRoute: null }];
    }

    let directRoute = null;
    if (device.directUrl) {
      const cached = probeCache.get(device.nodeId);
      if (!refresh && cached?.directUrl === device.directUrl && now() - cached.checkedAt < probeCacheMilliseconds) {
        directRoute = cached.route;
      } else {
        const directBase = device.directUrl.endsWith('/') ? device.directUrl : `${device.directUrl}/`;
        const startedAt = performance.now();
        try {
          const response = await fetchImpl(`${directBase}api/direct/health`, {
            cache: 'no-store',
            credentials: 'omit',
            mode: 'cors',
            redirect: 'error',
            signal: AbortSignal.timeout(2500),
          });
          if (response.ok) {
            let cache = { configured: false, reachable: false };
            try {
              const health = await response.json();
              cache = {
                configured: health.artworkCacheConfigured === true,
                reachable: health.artworkCacheReachable === true,
              };
            } catch (_) { /* Older nodes return an empty successful health response. */ }
            directRoute = {
              nodeId: device.nodeId,
              base: directBase.replace(/\/$/, ''),
              latencyMs: performance.now() - startedAt,
              artworkCacheConfigured: cache.configured,
              artworkCacheReachable: cache.reachable,
            };
          }
        } catch (_) {
          directRoute = null;
        }
        probeCache.set(device.nodeId, {
          directUrl: device.directUrl,
          checkedAt: now(),
          route: directRoute,
        });
      }
    }

    const status = directRoute ? 'direct' : device.relayAvailable ? 'relay' : 'offline';
    return [device.nodeId, { nodeId: device.nodeId, status, directRoute }];
  }));
  return new Map(results);
}
