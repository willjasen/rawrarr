import assert from 'node:assert/strict';
import test from 'node:test';
import { decodeOptions } from '../../rust-arr/src/passkey-client.mjs';

const encoded = value => Buffer.from(value).toString('base64url');

test('decodes the nested WebAuthn publicKey response shape', () => {
  const options = decodeOptions({
    publicKey: {
      challenge: encoded('challenge'),
      user: { id: encoded('user-id'), name: 'owner' },
      allowCredentials: [{ id: encoded('credential-id'), type: 'public-key' }],
      rpId: 'localhost',
    },
  });

  assert.deepEqual([...options.challenge], [...Buffer.from('challenge')]);
  assert.deepEqual([...options.user.id], [...Buffer.from('user-id')]);
  assert.deepEqual([...options.allowCredentials[0].id], [...Buffer.from('credential-id')]);
  assert.equal(options.rpId, 'localhost');
});

test('reports incomplete server options without a replace TypeError', () => {
  assert.throws(
    () => decodeOptions({}),
    { message: 'The server returned incomplete passkey options. Reload the page and try again.' },
  );
  assert.throws(
    () => decodeOptions({ publicKey: {} }),
    { message: 'The server returned incomplete passkey options. Reload the page and try again.' },
  );
});
