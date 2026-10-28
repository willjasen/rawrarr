import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const compose = await readFile(new URL('../../deploy/do3/docker-compose.public-ui-updater.yml', import.meta.url), 'utf8');

test('public coordinator receives the protected UI updater token and persistent UI bundle volume', () => {
  assert.match(compose, /services:\n  roarlay:/);
  assert.match(compose, /RAWARR_UI_GITHUB_REPO: willjasen\/rawrarr/);
  assert.match(compose, /RAWARR_UI_GITHUB_TOKEN_FILE: \/run\/secrets\/rawrarr_ui_github_token/);
  assert.match(compose, /- rawrarr_ui_github_token/);
  assert.match(compose, /- rawrarr-roarlay-prod-ui:\/ui/);
  assert.match(compose, /name: rawrarr-roarlay-prod-ui/);
  assert.match(compose, /RAWARR_UI_GITHUB_TOKEN_PATH:-\/root\/gh-token/);
  assert.doesNotMatch(compose, /\/shared|\/media|\btoken\s*:/i);
});
