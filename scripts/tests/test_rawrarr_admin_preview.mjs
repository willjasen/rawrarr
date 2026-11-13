import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const read = (path) => readFile(new URL(`../../${path}`, import.meta.url), "utf8");

test("admin preview is paused with no Docker build or launch definition", async () => {
  const architecture = await read("docs/admin-console.md");
  await Promise.all([
    assert.rejects(readFile(new URL("../../admin/Dockerfile", import.meta.url)), { code: "ENOENT" }),
    assert.rejects(readFile(new URL("../../docker-compose.admin.yml", import.meta.url)), { code: "ENOENT" })
  ]);
  assert.match(architecture, /preview is paused/i);
  assert.doesNotMatch(architecture, /docker compose .* up/i);
});

test("retained admin mockup source is sample-only and has no production data connection", async () => {
  const [html, script, nginx] = await Promise.all([
    read("admin/index.html"),
    read("admin/app.js"),
    read("admin/nginx.conf")
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
