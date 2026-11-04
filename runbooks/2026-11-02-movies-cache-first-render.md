# Movies cache first render

Issue: [#322](https://github.com/willjasen/rawrarr/issues/322)

## Change

On a refresh of the Movies view, restore the previously selected node from
`sessionStorage` and render its age-ID-gated movie text cache before waiting for
node health and route checks. Once node details load, refresh the library from
the node as before.

## Adoption and compatibility

No persistent data format, database schema, encrypted configuration, or node
protocol changes. Existing browser cache entries and session selection are
read as-is; users without a saved age ID or cached library continue through the
existing unlock flow. No migration is required. Deployment is the normal
coordinator application update.

## Verification

- PR [#324](https://github.com/willjasen/rawrarr/pull/324) merged to GitHub `main` as `b9ed69b46d514b612205619d19cb491034e9e91d`.
- Downloaded the archive for that exact published commit on do3 and verified SHA-256 `2ec818fd74918aa7eada0213738e41a251a30636c3efe149e5ee6db13f6ba9ef` before extracting and building from `rust-arr/`.
- Built native `linux/amd64` image `rawrarr:b9ed69b-amd64`, image ID `sha256:f2024e1e1186b745e6b135df04ab89d28d24030b0bc3f27f9d6494abf17ab2cf`, with the expected full source revision label. Promoted it to `rawrarr:internal-base-amd64` after preserving the previous base (`sha256:a1a076f430c29e414485c28f99e9db2d773dfc0cb9003a61d04aca12018da7bb`) and deployed service image (`sha256:c88f17f53f83e5a47d37681afbd36a7d6a43c86bbb60991682591c82af617485`) under rollback tags.
- Recreated only `rawrarr-internal-do3-roarlay` using the existing `rawrarr-internal-do3` Compose project, local image override with `pull_policy: never`, `--no-deps`, and `--pull never`. The service is healthy with zero restarts and reports the expected image ID, revision, and architecture. Local port 7299 health and HTTPS `/health` both identify roarlay.
- Tailscale Serve readback confirms `rawrarr.risk-mermaid.ts.net:443` `/` proxies to `http://127.0.0.1:8080`. The pre-deployment foreground Serve process pointed to port 8081; it was stopped and only the required HTTPS root handler was restored. No other handler was changed.
- The per-container roarlay lock was removed after verification. No database or subscriber migration was needed. Automated tests were not run; `git diff --check` passed before the initial commit.
- The shared local `main` checkout still has unrelated dirty edits overlapping paths changed upstream, so it was left untouched. Deployment used the exact authenticated GitHub archive for the merged commit, not local checkout files.
