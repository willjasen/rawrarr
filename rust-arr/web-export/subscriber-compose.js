export function subscriberCompose({ role, nodeId, coordinatorUrl, mediaPath = '' }) {
  if (!['metadata', 'storage'].includes(role)) throw new Error('Only metadata and apex nodes can be installed here.');
  if (!/^device-[0-9a-f]{24}$/.test(nodeId)) throw new Error('Invalid generated device ID.');
  let origin;
  try { origin = new URL(coordinatorUrl); }
  catch { throw new Error('The coordinator must have an HTTPS origin.'); }
  if (origin.protocol !== 'https:' || origin.origin !== coordinatorUrl || origin.username || origin.password) throw new Error('The coordinator must have an HTTPS origin.');
  if (role === 'storage' && (!/^(?:\/[^\0-\x1f]*|[A-Za-z]:[\\/][^\0-\x1f]+)$/.test(mediaPath.trim()))) throw new Error('Enter the absolute path to your media folder.');
  const quote = value => JSON.stringify(value);
  const lines = [
    '# Rawrarr subscriber node. Requires the rawrarr:local image until a public release is published.',
    '# This file runs no coordinator, roarlay, or PostgreSQL service.',
    'services:',
    `  ${role === 'storage' ? 'apex' : 'metadata'}:`,
    '    image: rawrarr:local',
    '    environment:',
    `      RAWARR_ROLE: ${role}`,
    '      RUST_ARR_LISTEN_ADDR: 0.0.0.0:8080',
    '      RAWARR_DATA_DIR: /data',
    '      RAWARR_SHARED_DATA_DIR: /shared',
    '      RAWARR_METADATA_DIR: /metadata',
    `      RAWARR_NODE_ID: ${nodeId}`,
    `      RAWARR_DEVICE_ID: ${nodeId}`,
    `      RAWARR_DEVICE_NAME: ${quote(role === 'storage' ? 'My apex node' : 'My metadata node')}`,
    `      RAWARR_COORDINATOR_URL: ${quote(coordinatorUrl)}`,
    '      RAWARR_NODE_TOKEN_FILE: /data/node-token',
    '      RAWARR_NODE_ENROLLMENT_ENABLED: "true"',
    '      RAWARR_WEBAUTHN_RP_ID: localhost',
    '      RAWARR_WEBAUTHN_ORIGIN: http://localhost:7297',
    '      RAWARR_LOCAL_PLAINTEXT_CONFIG: "false"',
  ];
  if (role === 'storage') lines.push('      RAWARR_LIBRARY_PATH: /media', `      RAWARR_DIRECT_ALLOWED_ORIGIN: ${quote(coordinatorUrl)}`);
  lines.push(
    '    ports:',
    '      - "127.0.0.1:7297:8080"',
    '    volumes:',
    '      - rawrarr-data:/data',
    '      - rawrarr-shared:/shared',
  );
  if (role === 'storage') lines.push('      - type: bind', `        source: ${quote(mediaPath.trim())}`, '        target: /media', '        read_only: true', '        bind:', '          create_host_path: false');
  lines.push('    restart: unless-stopped', 'volumes:', '  rawrarr-data:', '  rawrarr-shared:', '');
  return lines.join('\n');
}
