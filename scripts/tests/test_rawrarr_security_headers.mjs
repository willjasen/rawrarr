import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const source = await readFile(new URL('../../rust-arr/src/main.rs', import.meta.url), 'utf8');
const index = await readFile(new URL('../../scripts/tests/README.md', import.meta.url), 'utf8');

test('Rawrarr app adds a restrictive CSP and MIME protection to every response', async () => {
  assert.match(source, /async fn main_security_headers\(request: Request<Body>, next: Next\) -> Response<Body> \{[\s\S]*?CONTENT_SECURITY_POLICY[\s\S]*?default-src 'self'; base-uri 'none'; object-src 'none'; frame-ancestors 'none'; form-action 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:\/\/www\.themoviedb\.org; connect-src 'self' https: wss:; font-src 'self' data:; upgrade-insecure-requests[\s\S]*?X_CONTENT_TYPE_OPTIONS[\s\S]*?nosniff/);
  assert.match(source, /\.layer\(middleware::from_fn\(main_security_headers\)\)\s*\.with_state\(state\)/);
  const coordinator = await readFile(new URL('../../rust-arr/src/coordinator.rs', import.meta.url), 'utf8');
  assert.match(coordinator, /\.merge\(authenticated\)\s*\.layer\(middleware::from_fn\(crate::main_security_headers\)\)/);
  assert.match(index, /test_rawrarr_security_headers\.mjs/);
});
