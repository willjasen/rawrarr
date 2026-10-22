export async function deriveNodeAvatarSeed(nodeId, fetcher = fetch, cryptoApi = crypto) {
  if (typeof nodeId !== 'string' || !nodeId) throw new Error('Node ID is unavailable.');
  const response = await fetcher(`/nodes/${encodeURIComponent(nodeId)}/api/metadata/export/node-recipient`, {
    cache: 'no-store',
    headers: { Accept: 'text/plain' },
  });
  if (!response.ok) throw new Error('Node age recipient is unavailable.');
  const recipient = (await response.text()).trim();
  if (!/^age1pq1[a-z0-9]+$/.test(recipient)) throw new Error('Node returned an invalid age recipient.');
  const digest = new Uint8Array(await cryptoApi.subtle.digest('SHA-256', new TextEncoder().encode(recipient)));
  return Array.from(digest, byte => byte.toString(16).padStart(2, '0')).join('');
}
