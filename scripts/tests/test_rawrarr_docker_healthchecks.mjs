import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const dockerfile = await readFile(new URL('../../rust-arr/Dockerfile', import.meta.url), 'utf8');
const compose = await readFile(new URL('../../docker-compose.yml', import.meta.url), 'utf8');

test('Rawrarr and PostgreSQL containers define readiness health checks', () => {
  assert.match(dockerfile, /apt-get install[^\n]*\bcurl\b/);
  assert.match(dockerfile, /HEALTHCHECK --interval=10s --timeout=3s --start-period=15s --retries=5/);
  assert.match(dockerfile, /http:\/\/127\.0\.0\.1:8080\/health/);
  assert.match(compose, /postgres:[\s\S]*?healthcheck:[\s\S]*?pg_isready -U rawrarr -d rawrarr/);
});
