export function createCoordinatorAvailabilityMonitor({
  healthUrl = '/health',
  intervalMs = 3000,
  requestTimeoutMs = 4000,
  failuresBeforeUnavailable = 2,
  successesBeforeAvailable = 2,
  fetchImpl = globalThis.fetch.bind(globalThis),
  windowRef = globalThis.window,
  documentRef = globalThis.document,
  onUnavailable = () => {},
  onAvailable = () => {},
}) {
  let failures = 0;
  let successes = 0;
  let unavailable = false;
  let inFlight = false;
  let started = false;
  let intervalId;

  const check = async () => {
    if (inFlight) return;
    inFlight = true;
    const controller = new AbortController();
    const timeoutId = windowRef.setTimeout(() => controller.abort(), requestTimeoutMs);
    try {
      const response = await fetchImpl(healthUrl, {
        cache: 'no-store',
        credentials: 'same-origin',
        signal: controller.signal,
      });
      if (!response.ok) throw new Error(`Coordinator health returned HTTP ${response.status}`);
      failures = 0;
      if (unavailable) {
        successes += 1;
        if (successes >= successesBeforeAvailable) {
          unavailable = false;
          successes = 0;
          onAvailable();
        }
      }
    } catch (error) {
      successes = 0;
      failures += 1;
      if (!unavailable && failures >= failuresBeforeUnavailable) {
        unavailable = true;
        onUnavailable(error);
      }
    } finally {
      windowRef.clearTimeout(timeoutId);
      inFlight = false;
    }
  };

  const checkWhenVisible = () => {
    if (!documentRef.hidden) void check();
  };

  const start = () => {
    if (started) return;
    started = true;
    intervalId = windowRef.setInterval(checkWhenVisible, intervalMs);
    windowRef.addEventListener('online', checkWhenVisible);
    documentRef.addEventListener('visibilitychange', checkWhenVisible);
    void check();
  };

  const stop = () => {
    if (!started) return;
    started = false;
    windowRef.clearInterval(intervalId);
    windowRef.removeEventListener('online', checkWhenVisible);
    documentRef.removeEventListener('visibilitychange', checkWhenVisible);
  };

  return {
    start,
    stop,
    check,
    get unavailable() { return unavailable; },
  };
}
