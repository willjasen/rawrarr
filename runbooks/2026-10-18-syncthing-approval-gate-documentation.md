# Clarify Syncthing approval-gated onboarding

## Objective

Align current Rawrarr documentation with the production rule that new storage and metadata nodes must receive explicit owner approval before Syncthing peers or folders are configured. The two-Mac relay pairing is an operational test, not a production onboarding template.

## Documentation changes

- Clarified in the Mini storage-node plan that an empty local folder or installed Syncthing container is not permission to pair; no peer, shared folder, or copied shared data is allowed before approval.
- Clarified the owner-approval boundary in the data-domain and device-architecture docs: enrollment, Tailscale membership, and network reachability do not grant `/shared` access.
- Added the policy and public relay fallback status to technical status.
- Added this runbook to the runbook index.

## Verification

Reviewed current deployment instructions to ensure the order is enrollment, explicit owner approval, then deliberate peer/folder configuration. `git diff --check` and documentation consistency searches passed. This was a documentation-only update; no node or Syncthing runtime configuration changed.

## Remaining work

Automatic post-approval Syncthing provisioning is not implemented. Any future automation must verify the approved Rawrarr node identity and intended Syncthing device IDs before sharing the folder.
