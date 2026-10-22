export function subscriberCompose({ role, nodeId, coordinatorUrl, deviceName, mediaPath = '', networkMode, controlServer }) {
  if (!['metadata', 'storage'].includes(role)) throw new Error('Only fossil and boneyard nodes can be installed here.');
  if (!['internal-tailscale', 'headscale'].includes(networkMode)) throw new Error('Choose internal Tailscale or public Headscale networking.');
  if (!/^device-[0-9a-f]{24}$/.test(nodeId)) throw new Error('Invalid generated device ID.');
  const defaultName = role === 'storage' ? 'My boneyard node' : 'My fossil node';
  const name = String(deviceName ?? defaultName).trim();
  if (!name || name.length > 64 || /[\u0000-\u001f\u007f]/.test(name)) throw new Error('Enter a device name of 1 to 64 characters without control characters.');
  let origin;
  try { origin = new URL(coordinatorUrl); }
  catch { throw new Error('The coordinator must have an HTTPS origin.'); }
  if (origin.protocol !== 'https:' || origin.username || origin.password || origin.pathname !== '/' || origin.search || origin.hash) throw new Error('The coordinator must have an HTTPS origin.');
  coordinatorUrl = origin.origin;
  if (role === 'storage' && (!/^(?:\/[^\0-\x1f]*|[A-Za-z]:[\\/][^\0-\x1f]+)$/.test(mediaPath.trim()))) throw new Error('Enter the absolute path to your media folder.');
  const quote = value => JSON.stringify(value);
  const lines = [
    '# Rawrarr subscriber node. Requires the rawrarr:local image until a public release is published.',
    '# This file runs no coordinator, roarlay, or PostgreSQL service.',
    'services:',
    '  tailscale:',
    '    image: tailscale/tailscale:stable',
    `    hostname: ${quote(name.toLowerCase().replace(/[^a-z0-9-]+/g, '-').replace(/^-+|-+$/g, '').slice(0, 63) || nodeId)}`,
    '    environment:',
    '      TS_STATE_DIR: /var/lib/tailscale',
    '      TS_AUTHKEY: file:/run/secrets/tailscale-authkey',
    '      TS_AUTH_ONCE: "true"',
    '      TS_USERSPACE: "true"',
  ];
  const tsArgs = ['--accept-dns=false'];
  if (networkMode === 'headscale') {
    let loginServer;
    try { loginServer = new URL(controlServer); }
    catch { throw new Error('Enter the HTTPS Headscale control server URL.'); }
    if (loginServer.protocol !== 'https:' || loginServer.username || loginServer.password || loginServer.pathname !== '/' || loginServer.search || loginServer.hash) throw new Error('Enter the HTTPS Headscale control server URL.');
    tsArgs.push(`--login-server=${loginServer.origin}`);
  }
  lines.push(`      TS_EXTRA_ARGS: ${quote(tsArgs.join(' '))}`);
  lines.push(
    '    secrets:',
    '      - tailscale-authkey',
    '    volumes:',
    '      - tailscale-state:/var/lib/tailscale',
    '    restart: unless-stopped',
    '    entrypoint: ["/bin/sh", "-c"]',
    '    command: "/usr/local/bin/containerboot; result=$$?; rm -f /run/secrets/tailscale-authkey; exit $$result"',
    `  ${role === 'storage' ? 'boneyard' : 'fossil'}:`,
    '    image: rawrarr:local',
    '    environment:',
    `      RAWARR_ROLE: ${role}`,
    '      RUST_ARR_LISTEN_ADDR: 0.0.0.0:8080',
    '      RUST_ARR_DIRECT_LISTEN_ADDR: 0.0.0.0:8082',
    '      RAWARR_DATA_DIR: /data',
    '      RAWARR_SHARED_DATA_DIR: /shared',
    '      RAWARR_METADATA_DIR: /metadata',
    `      RAWARR_NODE_ID: ${nodeId}`,
    `      RAWARR_DEVICE_ID: ${nodeId}`,
    `      RAWARR_DEVICE_NAME: ${quote(name)}`,
    `      RAWARR_COORDINATOR_URL: ${quote(coordinatorUrl)}`,
    `      RAWARR_DIRECT_ALLOWED_ORIGIN: ${quote(coordinatorUrl)}`,
    '      RAWARR_NODE_TOKEN_FILE: /data/node-token',
    '      RAWARR_NODE_ENROLLMENT_ENABLED: "true"',
    '      RAWARR_WEBAUTHN_RP_ID: localhost',
    '      RAWARR_WEBAUTHN_ORIGIN: http://localhost:7297',
    '      RAWARR_LOCAL_PLAINTEXT_CONFIG: "false"',
  );
  if (role === 'storage') lines.push('      RAWARR_LIBRARY_PATH: /media');
  lines.push(
    '    network_mode: service:tailscale',
    '    depends_on:',
    '      tailscale:',
    '        condition: service_started',
    '    volumes:',
    '      - rawrarr-data:/data',
    '      - rawrarr-shared:/shared',
  );
  if (role === 'storage') lines.push('      - type: bind', `        source: ${quote(mediaPath.trim())}`, '        target: /media', '        read_only: true', '        bind:', '          create_host_path: false');
  lines.push(
    '    restart: unless-stopped',
    'secrets:',
    '  tailscale-authkey:',
    '    file: ./tailscale-authkey',
    'volumes:',
    '  tailscale-state:',
    '  rawrarr-data:',
    '  rawrarr-shared:',
    '',
  );
  return lines.join('\n');
}
