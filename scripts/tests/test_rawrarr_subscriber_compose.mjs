import assert from 'node:assert/strict';
import test from 'node:test';
import { subscriberCompose } from '../../rust-arr/web-export/subscriber-compose.js';

const base = { nodeId: `device-${'a'.repeat(24)}`, coordinatorUrl: 'https://rawrarr.example.test' };

test('subscriber Compose offers metadata and apex without coordinator services or secrets', () => {
  for (const role of ['metadata', 'storage']) {
    const yaml = subscriberCompose({ ...base, role, mediaPath: '/srv/media' });
    assert.match(yaml, new RegExp(`RAWARR_ROLE: ${role}`));
    assert.match(yaml, /127\.0\.0\.1:7297:8080/);
    assert.match(yaml, /rawrarr-data:\/data/);
    assert.match(yaml, /rawrarr-shared:\/shared/);
    assert.match(yaml, /RAWARR_NODE_TOKEN_FILE: \/data\/node-token/);
    assert.doesNotMatch(yaml, /^\s*(roarlay|coordinator|postgres):/m);
    assert.doesNotMatch(yaml, /AGE-SECRET|PRIVATE KEY|encryptedBackup|password/i);
  }
  const apex = subscriberCompose({ ...base, role: 'storage', mediaPath: '/srv/media' });
  assert.match(apex, /source: "\/srv\/media"[\s\S]*read_only: true/);
  assert.doesNotMatch(subscriberCompose({ ...base, role: 'metadata' }), /target: \/media/);
  assert.throws(() => subscriberCompose({ ...base, role: 'coordinator' }), /Only metadata and apex/);
  assert.throws(() => subscriberCompose({ ...base, role: 'storage' }), /media folder/);
  assert.throws(() => subscriberCompose({ ...base, role: 'storage', mediaPath: '../media' }), /media folder/);
  assert.throws(() => subscriberCompose({ ...base, role: 'metadata', coordinatorUrl: 'https://rawrarr.example.test/path' }), /HTTPS origin/);
  assert.match(subscriberCompose({ ...base, role: 'storage', mediaPath: 'C:/Media' }), /source: "C:\/Media"/);
});
