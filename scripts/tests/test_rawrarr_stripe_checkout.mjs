import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [billing, coordinator, compose, workspace, script] = await Promise.all([
  read('rust-arr/src/coordinator_stripe.rs'),
  read('rust-arr/src/coordinator.rs'),
  read('deploy/do3/docker-compose.stripe-checkout.yml'),
  read('rust-arr/src/coordinator_workspace.html'),
  read('rust-arr/src/coordinator-workspace.mjs'),
]);

test('checkout uses only a configured test key and validates the temporary one-dollar monthly price', () => {
  assert.match(billing, /secret_key\.starts_with\("sk_test_"\)/);
  assert.match(billing, /price\.get\("livemode"\)[\s\S]*?Some\(false\)/);
  assert.match(billing, /price\.get\("unit_amount"\)[\s\S]*?Some\(100\)/);
  assert.match(billing, /price\.get\("currency"\)[\s\S]*?Some\("usd"\)/);
  assert.match(billing, /Some\("month"\)/);
  assert.match(billing, /Some\(1\)\)/);
  assert.match(billing, /RAWARR_BILLING_AUDIENCE"\)\.is_ok_and\(\|value\| value == "public"\)/);
});

test('authenticated billing routes derive ownership from the session and protect POSTs with Origin', () => {
  assert.match(coordinator, /\.route\("\/api\/workspace\/billing", get\(stripe::status\)\)/);
  assert.match(coordinator, /\.route\("\/api\/workspace\/billing\/checkout", post\(stripe::checkout\)\)/);
  assert.match(coordinator, /\.route\("\/api\/workspace\/billing\/portal", post\(stripe::portal\)\)/);
  assert.match(billing, /authenticated_user_id\(&auth, &headers\)/);
  assert.match(billing, /\.get\(header::ORIGIN\)/);
  assert.match(billing, /metadata\[rawrarr_account_id\]/);
  assert.match(billing, /subscription_data\[metadata\]\[rawrarr_account_id\]/);
  assert.match(billing, /rawrarr_stripe_customers/);
  assert.match(billing, /rawrarr_stripe_subscriptions/);
});

test('do3 config mounts test billing credentials from host files and UI discloses no entitlement effect', () => {
  assert.match(compose, /STRIPE_TEST_SECRET_KEY_FILE: \/run\/secrets\/stripe-test-secret-key/);
  assert.match(compose, /STRIPE_TEST_PRICE_ID_FILE: \/run\/secrets\/stripe-test-price-id/);
  assert.match(compose, /file: \/opt\/rawrarr-roarlay-prod\/secrets\/stripe-test-secret-key/);
  assert.match(compose, /file: \/opt\/rawrarr-roarlay-prod\/secrets\/stripe-test-price-id/);
  assert.match(workspace, /temporary \$1\/month USD subscription through Stripe Checkout/);
  assert.match(workspace, /does not change Rawrarr access or entitlements/i);
  assert.match(script, /billing-status/);
});
