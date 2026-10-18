# Restore the coordinator sidebar logo

## Objective

Show the Rawrarr logo in the coordinator's left navigation menu.

## Initial state

The coordinator workspace rendered the word `rawrarr` as plain text. The main dashboard already used the Rawrarr logo image assets. The worktree contained extensive unrelated pre-existing changes.

## Actions

- Replaced the coordinator sidebar's text brand with the existing JXL/PNG responsive logo picture and added narrow-sidebar sizing.
- Added coordinator routes for the embedded PNG and JXL logo assets, which its standalone router did not previously serve.
- Fixed two compile errors encountered in existing coordinator work: imported `HeaderValue` in the node auth gateway and passed the serialized origin to direct-target validation.
- Rebuilt only the coordinator container under the existing `rawrarr-local` Compose project. PostgreSQL and Tailscale remained running.

## Verification

- The coordinator container reported `running` and `healthy`.
- `https://rawrarr.risk-mermaid.ts.net/health` returned HTTP 200 and the coordinator health page.
- Tailscale Serve status still showed `rawrarr.risk-mermaid.ts.net:443` `/` proxying to `http://127.0.0.1:8080`.
- Both `/assets/rawrarr-logo.png` and `/assets/rawrarr-logo.jxl` returned HTTP 200 with their expected image content types.

## Final state

The coordinator UI now serves its logo in the left navigation menu. The container rebuild lock was removed after verification. Existing unrelated worktree changes remain uncommitted and untouched.
