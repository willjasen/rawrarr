import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const projectRoot = fileURLToPath(new URL('../..', import.meta.url));
const ignoreScript = join(projectRoot, 'netlify-ignore-build.sh');

function runGit(cwd, ...args) {
  const result = spawnSync('git', args, { cwd, encoding: 'utf8' });
  assert.equal(result.status, 0, result.stderr || `git ${args.join(' ')} failed`);
  return result.stdout.trim();
}

function shouldIgnore(cwd, cachedCommit, commit) {
  const result = spawnSync('/bin/sh', [ignoreScript], {
    cwd,
    encoding: 'utf8',
    env: { ...process.env, CACHED_COMMIT_REF: cachedCommit, COMMIT_REF: commit },
  });
  return result.status;
}

test('Netlify ignores unrelated Git changes and builds for landing-page or config changes', async t => {
  const netlify = await readFile(join(projectRoot, 'netlify.toml'), 'utf8');
  assert.match(netlify, /ignore\s*=\s*"\.\/netlify-ignore-build\.sh"/);

  const temp = await mkdtemp(join(tmpdir(), 'rawrarr-netlify-ignore-'));
  t.after(() => rm(temp, { recursive: true, force: true }));
  runGit(temp, 'init', '-q');
  runGit(temp, 'config', 'user.email', 'test@example.invalid');
  runGit(temp, 'config', 'user.name', 'Rawrarr test');
  await mkdir(join(temp, 'website'), { recursive: true });
  await mkdir(join(temp, 'rust-arr'), { recursive: true });
  await writeFile(join(temp, 'website/index.html'), 'landing v1');
  await writeFile(join(temp, 'netlify.toml'), '[build]\npublish = "website"\n');
  await writeFile(join(temp, 'netlify-ignore-build.sh'), await readFile(ignoreScript));
  await writeFile(join(temp, 'rust-arr/src.rs'), 'backend v1');
  runGit(temp, 'add', '.');
  runGit(temp, 'commit', '-qm', 'base');
  const base = runGit(temp, 'rev-parse', 'HEAD');

  await writeFile(join(temp, 'rust-arr/src.rs'), 'backend v2');
  runGit(temp, 'commit', '-qam', 'backend-only');
  const backendOnly = runGit(temp, 'rev-parse', 'HEAD');
  assert.equal(shouldIgnore(temp, base, backendOnly), 0, 'backend-only changes are skipped');

  await writeFile(join(temp, 'website/index.html'), 'landing v2');
  runGit(temp, 'commit', '-qam', 'landing-change');
  const landingChange = runGit(temp, 'rev-parse', 'HEAD');
  assert.equal(shouldIgnore(temp, backendOnly, landingChange), 1, 'website changes trigger a build');

  await writeFile(join(temp, 'netlify.toml'), '[build]\npublish = "website"\n# updated config\n');
  runGit(temp, 'commit', '-qam', 'config-change');
  const configChange = runGit(temp, 'rev-parse', 'HEAD');
  assert.equal(shouldIgnore(temp, landingChange, configChange), 1, 'netlify.toml changes trigger a build');

  await writeFile(join(temp, 'netlify-ignore-build.sh'), `${await readFile(ignoreScript, 'utf8')}\n# changed helper`);
  runGit(temp, 'commit', '-qam', 'helper-change');
  const helperChange = runGit(temp, 'rev-parse', 'HEAD');
  assert.equal(shouldIgnore(temp, configChange, helperChange), 1, 'ignore helper changes trigger a build');
  assert.equal(shouldIgnore(temp, '', helperChange), 1, 'missing refs allow the build');
});
