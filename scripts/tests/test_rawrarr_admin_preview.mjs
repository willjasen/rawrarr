import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const read = (path) => readFile(new URL(`../../${path}`, import.meta.url), "utf8");

test("admin preview is explicitly sample-only and has no production data connection", async () => {
  const [html, script, nginx, compose, architecture] = await Promise.all([
    read("admin/index.html"),
    read("admin/app.js"),
    read("admin/nginx.conf"),
    read("docker-compose.admin.yml"),
    read("docs/admin-console.md")
  ]);

  assert.match(html, /Preview only — no live production data/);
  assert.match(html, /SAMPLE DATA/);
  assert.match(html, /Admin sign-in and live account reporting are planned/);
  assert.doesNotMatch(html, /\sstyle=/i);
  assert.match(script, /example\.com/);
  assert.doesNotMatch(script, /\.innerHTML\b/);
  assert.doesNotMatch(script, /\b(fetch|XMLHttpRequest|WebSocket|EventSource)\b/);
  assert.doesNotMatch(nginx, /\bproxy_pass\b/);
  assert.match(nginx, /connect-src 'none'/);
  assert.match(compose, /127\.0\.0\.1:7299:8090/);
  assert.doesNotMatch(compose, /network_mode:|depends_on:|^  tailscale:/m);
  assert.doesNotMatch(compose, /\b(postgres|DATABASE_URL|RAWARR_PG_)\b/i);
  assert.match(architecture, /require administrator authentication and authorization in the service/);
  assert.match(architecture, /do not enable Funnel/i);
});

test("sample account search and status filters render without a backend", async () => {
  const [html, script] = await Promise.all([
    read("admin/index.html"),
    read("admin/app.js")
  ]);

  assert.match(html, /id="user-search"/);
  assert.match(html, /id="status-filter"/);
  assert.match(html, /id="user-rows"/);
  assert.match(script, /search\.addEventListener\("input", renderUsers\)/);
  assert.match(script, /filter\.addEventListener\("change", renderUsers\)/);
  assert.match(script, /rows\.replaceChildren/);
});
