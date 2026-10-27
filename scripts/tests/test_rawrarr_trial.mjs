import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [auth, coordinator, migration, publicCompose, internalCompose, localCompose, workspace, script] = await Promise.all([
  read('rust-arr/src/auth.rs'),
  read('rust-arr/src/coordinator.rs'),
  read('rust-arr/migrations/0009_account_trials.sql'),
  read('docker-compose.coordinator.yml'),
  read('docker-compose.internal-mini.yml'),
  read('docker-compose.yml'),
  read('rust-arr/src/coordinator_workspace.html'),
  read('rust-arr/src/coordinator-workspace.mjs'),
]);

test('new accounts receive a three-day trial only for the public billing audience', () => {
  assert.match(auth, /fn public_billing_audience\(\) -> bool\s*\{\s*env::var\("RAWARR_BILLING_AUDIENCE"\)\.is_ok_and\(\|value\| value == "public"\)\s*\}/);
  assert.match(auth, /INSERT INTO rawrarr_users \(id, username, display_name, trial_started_at\) VALUES \(\$1, \$2, \$3, CASE WHEN \$4 THEN now\(\) ELSE NULL END\)/);
  assert.match(auth, /\.bind\(public_billing_audience\(\)\)/);
  assert.match(migration, /ADD COLUMN trial_started_at TIMESTAMPTZ;/);
  assert.doesNotMatch(migration, /DEFAULT|UPDATE rawrarr_users/i, 'legacy accounts must not receive a retroactive trial timestamp');
});

test('public coordinator defaults to public billing while internal Compose stacks opt out', () => {
  assert.match(publicCompose, /RAWARR_BILLING_AUDIENCE: \$\{RAWARR_BILLING_AUDIENCE:-public\}/);
  assert.match(internalCompose, /RAWARR_BILLING_AUDIENCE: internal/);
  assert.match(localCompose, /RAWARR_BILLING_AUDIENCE: internal/);
});

test('authenticated profile reports trial times only for public deployments', () => {
  assert.match(coordinator, /SELECT id, to_char\(trial_started_at AT TIME ZONE 'UTC'[\s\S]*?trial_started_at \+ interval '3 days'[\s\S]*?WHERE id=\$1/);
  assert.match(coordinator, /let trial_enabled = std::env::var\("RAWARR_BILLING_AUDIENCE"\)\.is_ok_and\(\|value\| value == "public"\)/);
  assert.match(coordinator, /trial_started_at: trial_enabled\.then_some\(trial_started_at\)\.flatten\(\)/);
  assert.match(coordinator, /trial_ends_at: trial_enabled\.then_some\(trial_ends_at\)\.flatten\(\)/);
});

test('profile displays the trial end in local time and distinguishes an expired trial', () => {
  assert.match(workspace, /id="trial-status"[^>]*hidden/);
  assert.match(script, /profile\.trialStartedAt && profile\.trialEndsAt/);
  assert.match(script, /endsAt\.toLocaleString\(\)/);
  assert.match(script, /Three-day trial ended/);
});
