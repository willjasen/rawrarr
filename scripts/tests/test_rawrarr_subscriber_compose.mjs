import assert from 'node:assert/strict';
import test from 'node:test';
import { subscriberCompose } from '../../rust-arr/web-export/subscriber-compose.js';

const base = { nodeId: `device-${'a'.repeat(24)}`, coordinatorUrl: 'https://rawrarr.example.test', networkMode: 'internal-tailscale', authKey: 'tskey-auth-test-fixture' };

test('subscriber Compose offers fossil and boneyard without coordinator services or secrets', () => {
  for (const role of ['metadata', 'storage']) {
    const yaml = subscriberCompose({ ...base, role, mediaPath: '/srv/media' });
    assert.match(yaml, new RegExp(`RAWARR_ROLE: ${role}`));
    assert.match(yaml, /network_mode: service:tailscale/);
    assert.match(yaml, /TS_AUTHKEY: "tskey-auth-test-fixture"/);
    assert.doesNotMatch(yaml, /tailscale-authkey|secrets:/);
    assert.match(yaml, /TS_HOSTNAME: rawrarr-my-fossil-node/);
    assert.match(yaml, /tailscale-state:\/var\/lib\/tailscale/);
    assert.match(yaml, /rawrarr-data:\/data/);
    assert.match(yaml, /rawrarr-shared:\/shared/);
    assert.match(yaml, /RAWARR_NODE_TOKEN_FILE: \/data\/node-token/);
    assert.match(yaml, /RAWARR_DIRECT_ALLOWED_ORIGIN: "https:\/\/rawrarr\.example\.test"/);
    assert.match(yaml, /RUST_ARR_DIRECT_LISTEN_ADDR: 0\.0\.0\.0:8082/);
    assert.doesNotMatch(yaml, /^\s*(roarlay|coordinator|postgres):/m);
    assert.doesNotMatch(yaml, /AGE-SECRET|PRIVATE KEY|encryptedBackup|password/i);
  }
  const boneyard = subscriberCompose({ ...base, role: 'storage', mediaPath: '/srv/media' });
  assert.match(boneyard, /  boneyard:\n/);
  assert.match(boneyard, /source: "\/srv\/media"[\s\S]*read_only: true/);
  assert.match(subscriberCompose({ ...base, role: 'metadata' }), /  fossil:\n/);
  assert.doesNotMatch(subscriberCompose({ ...base, role: 'metadata' }), /target: \/media/);
  assert.throws(() => subscriberCompose({ ...base, role: 'coordinator' }), /Only fossil and boneyard/);
  assert.throws(() => subscriberCompose({ ...base, role: 'storage' }), /media folder/);
  assert.throws(() => subscriberCompose({ ...base, role: 'storage', mediaPath: '../media' }), /media folder/);
  assert.throws(() => subscriberCompose({ ...base, role: 'metadata', coordinatorUrl: 'https://rawrarr.example.test/path' }), /HTTPS origin/);
  assert.match(subscriberCompose({ ...base, role: 'storage', mediaPath: 'C:/Media' }), /source: "C:\/Media"/);
  assert.match(subscriberCompose({ ...base, role: 'metadata', deviceName: 'Office fossil' }), /RAWARR_DEVICE_NAME: "Office fossil"/);
  assert.throws(() => subscriberCompose({ ...base, role: 'metadata', deviceName: '   ' }), /device name/);
  assert.throws(() => subscriberCompose({ ...base, role: 'metadata', deviceName: 'x'.repeat(65) }), /device name/);
  assert.throws(() => subscriberCompose({ nodeId: base.nodeId, coordinatorUrl: base.coordinatorUrl, role: 'metadata' }), /Choose internal Tailscale/);
});

test('public mode configures the selected Headscale control server without embedding its key', () => {
  const yaml = subscriberCompose({ ...base, role: 'metadata', networkMode: 'headscale', controlServer: 'https://headscale.example.test' });
  assert.match(yaml, /TS_EXTRA_ARGS: "--accept-dns=false --login-server=https:\/\/headscale\.example\.test"/);
  assert.match(yaml, /TS_AUTHKEY: file:\/run\/secrets\/tailscale-authkey/);
  assert.doesNotMatch(yaml, /TS_EXTRA_ARGS:.*tailscale\.com/);
  assert.throws(() => subscriberCompose({ ...base, role: 'metadata', networkMode: 'headscale' }), /Headscale control server URL/);
  assert.throws(() => subscriberCompose({ ...base, role: 'metadata', networkMode: 'headscale', controlServer: 'http://headscale.example.test' }), /HTTPS Headscale/);
});

test('internal mode uses the Mac mini Tailscale HTTPS coordinator origin', () => {
  const coordinatorUrl = 'https://rawrarr-mini.example-tailnet.ts.net';
  const yaml = subscriberCompose({ ...base, role: 'metadata', coordinatorUrl, networkMode: 'internal-tailscale' });
  assert.match(yaml, /RAWARR_COORDINATOR_URL: "https:\/\/rawrarr-mini\.example-tailnet\.ts\.net"/);
  assert.match(yaml, /RAWARR_DIRECT_ALLOWED_ORIGIN: "https:\/\/rawrarr-mini\.example-tailnet\.ts\.net"/);
  assert.match(yaml, /TS_HOSTNAME: rawrarr-my-fossil-node/);
  assert.match(yaml, /TS_AUTHKEY: "tskey-auth-test-fixture"/);
});
