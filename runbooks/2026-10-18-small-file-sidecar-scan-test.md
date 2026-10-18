# Small-file sidecar scan test

## Objective and initial state

Run the new Rust sidecar scan against a few 1–2 GB movie files on the Mac mini, then verify encrypted shared-folder output. The prior implementation task documented that its code had not yet been deployed to the Mini. The current worktree also contains substantial unrelated changes; none were altered for this live test.

## Read-only preflight

- The Mini tailnet hostname resolves to `100.90.20.11`; `https://rawrarr-mini.risk-mermaid.ts.net/health` responds HTTP 200 with an empty body.
- The HTTPS root and `/api/library` also respond from Caddy with HTTP 200 and zero-byte bodies, so they do not establish that the Rawrarr application or the new scanner is deployed.
- SSH as `willjasen` to the Mini on port 22 is refused. The `tailscale ssh` wrapper reaches the same refused port.
- The current scanner has no remote request for scanning a selected file set. Its regular library scan walks the configured root, so invoking it would not be limited to the requested few files.

## Result

No media scan was run, no media was read, and no files were written to the shared folder. A targeted sample test remains pending until remote execution is available and the Rust sidecar build is confirmed on the Mini. The test should identify a few files between 1 and 2 GB, scan only those files, decrypt the resulting `.json.aged` records with the authorized identity for inspection, and confirm ciphertext propagation in `/shared/metadata/library/text/`.

## Recovery

No recovery action is needed because the preflight made no changes.
