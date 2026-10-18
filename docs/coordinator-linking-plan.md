# Coordinator device linking plan

> Updated container architecture (2026-10-18): central **coordinator** owns passkeys/authentication, account ownership, the target primary UI, and node coordination; **metadata** and **storage** provide node-local capabilities. Only storage reads media. The MacBook metadata node currently serves the transitional UI; users should eventually reach it through the coordinator dashboard. Manual approval with account-scoped pending requests is implemented; browser device-code linking and multi-account invitations remain planned.

Status: the current MacBook/Mini enrollment protocol, routes, pending UI, and explicit approval are implemented. The device-code flow below remains the future multi-account invitation design.

## Roles and joining nodes

The coordinator is assumed to be already provisioned and healthy. The MacBook's existing **metadata** node is the seeded first participant; it connects to that coordinator and currently hosts the transitional browser UI. The intended user entry point is the coordinator-hosted dashboard, which will show both metadata and storage nodes without requiring `localhost:7297`. The Mac mini runs an OrbStack **storage** worker reading `/Volumes/silo/Media/data` through a read-only `/media` mount. It joins the existing instance, but does not receive user passkeys, sessions, PostgreSQL, or a private age identity merely to scan. The operator **service** is separate from all subscriber roles.

The seeded MacBook metadata node already has its approved-node credential and authenticated access to the established coordinator. It is not part of the joining workflow. Each later metadata or storage node creates a credential locally and sends no user ID; the coordinator binds the pending request to its existing owner and filters approval to that same signed-in owner. The current implementation rejects enrollment if the coordinator has no account or multiple accounts exist. Multi-account enrollment must use an owner-issued, short-lived invitation; the node must never choose its account by supplying a user ID.

## Owner experience

Use a device-code flow similar in interaction to GitHub/Tailscale device enrollment or Jellyfin Quick Connect. These are UX analogies, not protocol compatibility claims.

1. The worker is configured with the trusted coordinator HTTPS base URL and obtains network access independently. It creates and persists a device key locally, then registers a pending storage enrollment using proof of key possession.
2. After the coordinator accepts the pending request, the worker prints the linking URL and human-entered device code to its local Docker logs. Example shape only:

   ```text
   Link this storage node:
   Open https://rawrarr.<tailnet>.ts.net/devices/link
   Enter code: <short-lived-device-code>
   Waiting for approval. Code expires at <expiry>.
   ```

3. The owner opens that URL, signs in with a coordinator passkey, enters the code, and reviews the requested node name, storage capability, and device-key fingerprint. Untrusted device names must be escaped and clearly identified as reported by the device. Merely opening the page or entering a code does not approve the node.
4. The owner explicitly approves linking the device to the existing instance. The coordinator binds authorization to the pending device public key and scoped storage capability. The worker polls with proof of its own key, receives its enrollment result, and reports linked status. No browser session or user passkey moves to the worker.
5. Subsequent starts reuse the approved device identity. Expired or denied requests cannot become approved through replay. Revoked devices must stop authenticated operations and require explicit re-enrollment; they must not silently mint new identities to bypass revocation.

The fixed `/devices/link` path and displayed code are the selected interaction. Do not put an authorization token or code into the URL. Exact API routes, code alphabet/length, expiry duration, polling interval, and signature algorithm remain implementation choices.

## Enrollment contract

- The coordinator issues an unpredictable, short-lived, single-use human code scoped to its instance and one pending device public key. Uniqueness and entropy must be sufficient for the configured rate limits; do not derive it from a hostname or device ID.
- The code is intentionally visible in local Docker logs for the operator, but never in synchronized `/shared/logs`, request URLs, analytics, shared configuration, or source control. Keep durable device credentials, polling secrets, and private keys out of all logs. Audit the existing shared logger before adding this output.
- Store only a verifier for the human code, expire pending requests, and rate-limit both issuance and code attempts. Require a signed-in authorized owner and CSRF protection for approval. Generic invalid/expired responses should not enumerate pending devices.
- Bind the approval to the exact pending key, instance, requested capability, and expiry. Verify worker key possession using fresh, replay-resistant challenges. A bearer of the human code must not be able to retrieve or impersonate the worker's credentials.
- Poll with bounded backoff and explicit pending, approved, denied, expired, revoked, and coordinator-unreachable states. Duplicate polls or approvals must be idempotent. On expiry, request a fresh code explicitly and invalidate the old one.
- Validate the coordinator HTTPS URL and certificate. Do not follow enrollment redirects to arbitrary origins or disable TLS verification. The coordinator's trusted public URL generates the log URL; do not build it from an untrusted Host header. The URL is non-secret configuration; Tailscale credentials are separate.
- Linking does not automatically grant a full writable Syncthing replica. Provision only the approved app-data/inventory access after approval; keep the instance manifest and encryption recipient authoritative. Define scoped catalog uploads, heartbeat, and commands separately from browser session routes.
- Initial device-signature candidate is ML-DSA (NIST FIPS 204), pending implementation review. Enrollment should prove possession of the candidate private key and bind owner approval to its public key, derived device ID, and requested role. The coordinator keeps the approved public key in its protected registry for coordination authentication; the age-encrypted shared config replicates the approved public-key registry for peer verification of signed `/shared` records. Do not trust a key only because it appears in a pending request or in the config revision it claims to sign. The signing design note covers config trust bootstrap, signed record envelopes, and revocation. The instance trust root is still undecided: options include making the first device the root, using a distinct owner key held by the owner (with a proposed age-encrypted backup) or delegated to roarlay, or a threshold/recovery authority. One candidate is to create and back up the age identity first, generate the ML-DSA root locally, encrypt its backup to the age recipient, and instruct the owner to copy/save the ciphertext. For each later approval, the owner pastes that encrypted backup into the client; the client decrypts and signs locally, verifies the pinned root fingerprint, and sends only the signed approval. Instructions must explain backup retention and age-identity recovery. Roarlay must never receive the plaintext root key. If roarlay holds the per-instance authority key, its compromise could authorize malicious nodes/config changes; secret isolation, backup, rotation, and recovery must be designed. A global Rawrarr publisher key, if used for release authenticity, must remain separate from subscriber-instance authorization.

## Network and authentication boundary

The worker initiates HTTPS to the coordinator through Tailscale. Tailscale controls network reachability; Rawrarr enrollment controls application permissions. The code flow cannot bootstrap its own missing Tailscale membership or DNS reachability: establish those first, then link Rawrarr.

Prefer a stable named Tailscale Service for the coordinator, with only the MacBook coordinator as its initial backend. A device-specific Serve URL can be a first verification step. Storage and shard nodes must not advertise the coordinator UI as equivalent backends. Preserve the local app listener at `127.0.0.1:7297`; a host-level Serve proxy can target it. A sidecar must reach the app through an explicitly verified Docker network, not assume its own localhost is the Mac host.

The existing localhost passkey RP/origin is not automatically usable on a Tailscale HTTPS hostname. Before deployment, implement and test a controlled origin/RP enrollment or migration path while retaining working local access. Never broaden accepted origins arbitrarily or assume that TLS alone fixes WebAuthn. Do not publish the link as usable before login works at that URL.

Tailscale does not transfer media or expose PostgreSQL. Coordinator sleep/offline state means pending enrollment and commands wait; the worker retries safely and preserves bounded queued state. Replicated ciphertext alone does not make a shard a failover coordinator.


## Local age identity provisioning (planned)

The owner will provision the existing instance's age private ID directly on the Mac mini, separately from device linking. Plan a local setup command with hidden terminal input that writes an owner-only file outside the checkout, media, and synchronized folders. Do not supply the value in arguments, environment variables, shell history, Compose YAML, logs, or the coordinator/browser linking flow. The concrete command and secret path are not implemented yet.

Mount the protected file read-only into the worker through a local secret mount, readable only by its runtime user. Validate the identity locally against the approved instance recipient without printing either complete key. Reject mismatches without replacing the shared recipient or creating a new instance. The private ID remains on the Mini and is used only by explicitly authorized local decryption features; encryption-only scanning still needs only the public recipient. Key availability and device approval are independent: possessing the age key does not enroll the node, and linking does not transfer the key.

Verify absent/malformed/mismatched identity handling, mount permissions, persistence across restart, and absence from shared data and all logs. Define local replacement/removal and backup/recovery instructions before unattended deployment. Rawrarr device revocation stops authorized API operations but cannot revoke an age key already copied to a device; key compromise requires a separate instance encryption-key rotation plan.

## Implementation and acceptance

1. Separate persistent storage startup from web/auth/PostgreSQL. The existing `storage-scan-once` already exits before database connection; it is a foundation, not a running worker.
2. Implement pending enrollment persistence on the coordinator, device-key proof, local Docker log code output, owner code entry/approval, polling, and revocation.
3. Test wrong/expired/reused codes, rate limits, wrong owner/instance/key, unsigned requests, CSRF, approval races, lost replies, restart continuity, and revoked devices. Verify no private keys or long-lived credentials leak, and that human codes are absent from shared logs.
4. Verify public HTTPS reachability and authenticated linking from the actual Mac mini OrbStack worker container and from the owner's browser. Tailscale Serve is an optional alternate path to the coordinator, not a requirement for the Mini. A successful `/health` request proves only web responsiveness.
5. Scan a bounded read-only fixture, publish encrypted inventory, and verify the coordinator displays it for the approved device. A successful pairing or file sync alone is not catalog completion.

Related: [node types](node-types.md), [device signing](device-signing-design.md), [Tailscale integration](tailscale-integration-plan.md), and [Mac mini deployment](mac-mini-storage-node-plan.md).
