import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const dockerfile = await readFile(new URL('../../rust-arr/Dockerfile', import.meta.url), 'utf8');
const compose = await readFile(new URL('../../docker-compose.yml', import.meta.url), 'utf8');
const testRunner = await readFile(new URL('./run_rawrarr_tests_in_docker.py', import.meta.url), 'utf8');
const testEntrypoint = await readFile(new URL('./run-tests-in-container.sh', import.meta.url), 'utf8');
const testDockerfile = await readFile(new URL('./Dockerfile', import.meta.url), 'utf8');
const clonePreparation = await readFile(new URL('./prepare-test-clone.sh', import.meta.url), 'utf8');
const cloneEntrypoint = await readFile(new URL('./wait-for-test-trigger.sh', import.meta.url), 'utf8');
const syncthingTest = await readFile(new URL('./test_rawrarr_syncthing_orbstack.py', import.meta.url), 'utf8');

test('Rawrarr and PostgreSQL containers define readiness health checks', () => {
  assert.match(dockerfile, /apt-get install[^\n]*\bcurl\b/);
  assert.match(dockerfile, /HEALTHCHECK --interval=10s --timeout=3s --start-period=15s --retries=5/);
  assert.match(dockerfile, /http:\/\/127\.0\.0\.1:8080\/health/);
  assert.match(compose, /postgres:[\s\S]*?healthcheck:[\s\S]*?pg_isready -U rawrarr -d rawrarr/);
});

test('Rawrarr image builds its Rust binary for the requested container architecture', () => {
  assert.match(dockerfile, /ARG TARGETARCH/);
  assert.match(dockerfile, /amd64\) rust_target=x86_64-unknown-linux-gnu/);
  assert.match(dockerfile, /arm64\) rust_target=aarch64-unknown-linux-gnu/);
  assert.match(dockerfile, /Unsupported target architecture/);
  assert.match(dockerfile, /cargo build --release --locked --target "\$rust_target"/);
  assert.match(dockerfile, /test "\$\(cat \/usr\/share\/rawrarr\/target-arch\)" = "\$TARGETARCH"/);
});

test('test base installs age for the native target architecture', () => {
  assert.match(testDockerfile, /ARG TARGETARCH/);
  assert.match(testDockerfile, /amd64\) age_arch=amd64/);
  assert.match(testDockerfile, /arm64\) age_arch=arm64/);
  assert.match(testDockerfile, /6b8dc4333c53a5a57c9e5834e3a48f92605d7154014cd07269ff3327db5d37f4/);
  assert.match(testDockerfile, /test "\$\(dpkg --print-architecture\)" = "\$TARGETARCH"/);
});

test('Docker test containers have unique names in the rawrarr-test-regression group', () => {
  assert.match(testRunner, /BASE36 = "0123456789abcdefghijklmnopqrstuvwxyz"/);
  assert.match(testRunner, /def timestamp_code\(\)/);
  assert.match(testRunner, /value = time\.time_ns\(\) \/\/ 1_000_000/);
  assert.match(testRunner, /name = f"\{PROJECT\}-tests-\{run_id\}"/);
  assert.match(testRunner, /com\.docker\.compose\.service=tests-\{run_id\}/);
  assert.match(testRunner, /com\.docker\.compose\.project=\{PROJECT\}/);
  assert.match(testRunner, /if args\.rebuild_base or not image_exists:/);
  assert.match(testRunner, /copy_project_into_container\(args\.context, name/);
  assert.match(testRunner, /prepare-test-clone\.sh/);
  assert.match(testRunner, /network[\s\S]*disconnect[\s\S]*bridge/);
  assert.match(testRunner, /touch.*rawrarr-test-ready/);
  assert.match(clonePreparation, /cargo fetch --locked --manifest-path \/workspace\/rust-arr\/Cargo\.toml/);
  assert.match(clonePreparation, /test_rawrarr_movie_snapshot\.rs/);
  assert.match(clonePreparation, /dpkg --print-architecture/);
  assert.match(clonePreparation, /age-v1\.3\.2-linux-\$\{age_arch\}\.tar\.gz/);
  assert.match(cloneEntrypoint, /rawrarr-test-ready/);
  assert.match(testRunner, /shutil\.copytree/);
  assert.doesNotMatch(testRunner, /copytree\([^\n]*web-export/);
  assert.match(testRunner, /ROOT \/ "docs"/);
  assert.match(testRunner, /ROOT \/ "\.github"/);
  assert.match(testRunner, /ROOT \/ "stripe-service"/);
  assert.match(testEntrypoint, /grep -Eq.*playwright-core/);
  assert.match(testEntrypoint, /chromium.*launch/);
  assert.match(testEntrypoint, /puppeteer/);
  assert.match(testRunner, /f"\{rust_source\}\/\."/);
  assert.match(testRunner, /f"\{tests_source\}\/\."/);
  assert.match(testRunner, /--rebuild-base/);
  assert.match(testRunner, /--entrypoint", "\/bin\/sh"/);
  assert.match(cloneEntrypoint, /run-tests-in-container\.sh/);
  assert.doesNotMatch(testRunner, /build_context \/ "rust-arr"/);
  assert.match(testEntrypoint, /cargo clean --manifest-path \/workspace\/rust-arr\/Cargo\.toml -p rust-arr/);
  assert.match(testEntrypoint, /={16,}[\s\S]*CONTAINER TEST SUMMARY[\s\S]*={16,}/);
  assert.match(testEntrypoint, /section_start "NODE\.JS TESTS"/);
  assert.match(testEntrypoint, /section_start "PLAYWRIGHT BROWSER TESTS"/);
  assert.match(testEntrypoint, /section_start "RUST UNIT TESTS"/);
  assert.match(testEntrypoint, /section_start "PYTHON INTEGRATION TESTS"/);
  assert.match(testEntrypoint, /END %s \(status=%s\)/);
  assert.match(testEntrypoint, /print_result "Node\.js"/);
  assert.match(testEntrypoint, /node --experimental-global-webcrypto --test "\$browser_test" >>"\$playwright_log" 2>&1/);
  assert.match(testEntrypoint, /print_result "Playwright browser"/);
  assert.match(testEntrypoint, /TEST RESULTS playwright=%s node=%s rust=%s/);
  assert.match(testEntrypoint, /test_started_at=/);
  assert.match(testEntrypoint, /test_stopped_at=/);
  assert.match(testEntrypoint, /Test run:.*seconds/);
  assert.match(testRunner, /--cleanup-dry-run/);
  assert.match(testRunner, /CLEANUP_INTERVAL_SECONDS = 60 \* 60/);
  assert.match(testRunner, /\.test-container-cleanup-last-run/);
  assert.match(testEntrypoint, /print_result "Rust"/);
  assert.match(testEntrypoint, /not ok \[0-9\]\+/);
  assert.match(testEntrypoint, /\\\.\\\.\\\. FAILED/);
  assert.match(syncthingTest, /BASE36 = "0123456789abcdefghijklmnopqrstuvwxyz"/);
  assert.match(syncthingTest, /suffix = timestamp_code\(\)/);
  assert.match(syncthingTest, /names = \[f"\{PROJECT\}-web-\{suffix\}", f"\{PROJECT\}-storage-\{suffix\}"\]/);
  assert.match(syncthingTest, /services = \[f"web-\{suffix\}", f"storage-\{suffix\}"\]/);
});
