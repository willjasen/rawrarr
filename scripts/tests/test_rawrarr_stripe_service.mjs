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

test('Stripe migrations use an independent ledger from the account service', async () => {
  const handler = await source('stripe-service/src/main.rs');
  assert.match(handler, /CREATE TABLE IF NOT EXISTS rawrarr_stripe_migrations/);
  assert.match(handler, /SELECT checksum FROM rawrarr_stripe_migrations/);
  assert.match(handler, /raw_sql\(migration_sql\)/);
  assert.doesNotMatch(handler, /sqlx::migrate!/);
});

test('do3 webhook service stays private behind Caddy and file-based secrets', async () => {
  const compose = await source('deploy/do3/docker-compose.stripe-service.yml');
  const caddy = await source('deploy/do3/stripe-service.Caddyfile');
  assert.match(compose, /BILLING_DATABASE_PASSWORD_FILE: \/run\/secrets\/stripe-db-password/);
  assert.match(compose, /STRIPE_WEBHOOK_SECRET_FILE: \/run\/secrets\/stripe-webhook-secret/);
  assert.match(compose, /rawrarr-roarlay-prod-control/);
  assert.doesNotMatch(compose, /ports:/);
  assert.match(caddy, /path \/v1\/webhooks\/stripe/);
  assert.match(caddy, /reverse_proxy stripe-service:8090/);
});
