import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const root = new URL('../../', import.meta.url);
const source = async (path) => readFile(new URL(path, root), 'utf8');

test('Stripe billing rows link to the passkey-owning account table', async () => {
  const migration = await source('stripe-service/migrations/0002_stripe_account_link.sql');
  const handler = await source('stripe-service/src/main.rs');
  assert.match(migration, /account_id UUID PRIMARY KEY REFERENCES rawrarr_users\(id\)/);
  assert.match(migration, /account_id UUID NOT NULL REFERENCES rawrarr_users\(id\)/);
  assert.match(handler, /metadata\(\)\s*|rawrarr_account_id/);
  assert.match(handler, /SELECT account_id FROM rawrarr_stripe_customers WHERE stripe_customer_id=\$1/);
  assert.match(handler, /StatusCode::CONFLICT/);
  assert.match(handler, /ON CONFLICT\(stripe_subscription_id\) DO UPDATE/);
});

test('service persists no passkey payloads or raw webhook bodies', async () => {
  const handler = await source('stripe-service/src/main.rs');
  const readme = await source('stripe-service/README.md');
  assert.doesNotMatch(handler, /rawrarr_passkeys|passkey JSON|body\.to_vec\(\)/i);
  assert.match(readme, /does not read or write passkey records/);
  assert.match(readme, /never point it at a subscriber node's PostgreSQL instance/);
});

test('event inbox and account updates use one transaction', async () => {
  const handler = await source('stripe-service/src/main.rs');
  assert.match(handler, /INSERT INTO stripe_webhook_events[\s\S]*ON CONFLICT\(event_id\) DO NOTHING/);
  assert.match(handler, /transaction\.commit\(\)\.await/);
  assert.match(handler, /DefaultBodyLimit::max\(1_048_576\)/);
});
