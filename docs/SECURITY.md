# Rawrarr security

This document describes the security boundaries in Rawrarr, including which systems can see plaintext, how persistent data is encrypted, and how browser requests pass through the coordinator. It distinguishes implemented behavior from design goals that are not yet complete. See [container deployment](container-deployment.md), [data domains](data-domains.md), and [blind coordinator content flow](blind-coordinator-content.md) for operational and architecture details.

## Security goals and limits

Rawrarr separates account authentication and node coordination from subscriber content. A coordinator should learn only what it needs to authenticate a user, select an authorized node, route a request, and report operational state. Metadata and storage nodes handle subscriber data. The browser is the display endpoint and, for end-to-end encrypted content, the place where decryption occurs.

“Blind coordinator” does not mean anonymous or metadata-free. The coordinator necessarily sees account and node identifiers, roles, authorization state, route choices, timing, request sizes, source network information, and online status. Its database stores account/passkey/session/node coordination data. These facts can reveal that a user operates particular nodes and when they are active.

This document is not a claim of a completed independent security audit. The current implementation has gaps described below, including limited end-to-end browser verification and legacy node APIs that return plaintext when accessed directly by an authorized browser.

## Roles and trust boundaries

| Component | Intended responsibility | Sensitive information it can see |
| --- | --- | --- |
| Coordinator | Passkey authentication, node authorization, routing, and ciphertext relay | Account and node metadata, sessions, request metadata, and opaque ciphertext. It must not receive subscriber content plaintext or private age identities. |
| Metadata node | Connect to Radarr/Sonarr, prepare metadata, serve the app, encrypt shared content | Radarr/Sonarr responses, API credentials, library inventory, and plaintext needed to render data locally. |
| Storage node | Read only its locally mounted media when enabled, prepare sidecars, store shared ciphertext | Its local media paths and files, local inventory during scans, and ciphertext. It does not need a private age identity to encrypt to the public recipient. |
| User browser | Authenticate, request authorized content, decrypt and render locally | The user's passkey session and whatever plaintext is displayed. Age private identities and ephemeral transport private keys should remain here when used for browser decryption. |
| Syncthing peers | Replicate explicitly shared `/shared` application data | Age-encrypted config/metadata ciphertext, the public age recipient, and any other explicitly shared files. Peer/device identities and sync metadata are visible. |

The coordinator must never mount `/media`, `/shared`, metadata-node `/data`, storage-node `/data`, age private identities, or subscriber media. PostgreSQL remains coordinator-local and is not replicated with Syncthing. A node's approval, network reachability, or presence on a Tailscale network does not itself authorize access to a user's data.

## Authentication and authorization

- User sign-in uses WebAuthn passkeys. The coordinator stores public passkey credential data and authentication/session records; it does not need the user's age private identity.
- Coordinator sessions are opaque random tokens stored server-side by hash and returned in `HttpOnly`, `SameSite=Lax`, `Secure` cookies for HTTPS deployments. Logout expires the browser cookie and invalidates the session.
- Node credentials are random node-local bearer secrets. Enrollment sends public node details and a one-way verifier; new nodes remain pending until explicit owner approval. Node credentials and local node state must not be written to `/shared`, logs, or Git.
- Node-scoped delegated sessions bind access to an owner and a node. The coordinator checks ownership and role before relaying. Relay targets come from trusted server configuration; a browser cannot choose an arbitrary upstream URL.
- Authorization answers “may this account/node request the object?” Encryption answers “can this key decrypt the content?” Keep these controls independent. Possessing an age identity must not bypass account, node, or object authorization.

## Age encryption for stored and shared content

Rawrarr uses the `age` tools with post-quantum age identities (`age-keygen -pq`) for persistent shared configuration and metadata. The private identity is created for the user, remains on a subscriber device or in the user's browser when imported there, and is not sent to the coordinator. The matching public age recipient may be stored in shared data so authorized nodes, including storage-only nodes, can encrypt content without holding the private identity.

The metadata node encrypts `config.aged` and metadata files to the public recipient before writing them into `/shared`. Storage nodes can encrypt scan sidecars to the same recipient. Syncthing replicates those ciphertext files; it does not make them readable. Only a holder of a matching private identity can decrypt them. Loss of the private identity means the existing ciphertext cannot be recovered by the coordinator.

`metadata_filename_key` is a separate random 256-bit secret inside age-encrypted config. It derives domain-separated HMAC-SHA-256 filenames for movie text and artwork so a shared-folder observer cannot directly read TMDb IDs from filenames. It is not an encryption key, age identity, passkey, or authentication credential. Do not rotate it after files have been written without a migration plan.

Age protects content at rest and across shared-folder replication. It does not hide file sizes, modification times, approximate content counts, node participation, or sync timing. The public recipient is not secret. Some operational logs are allowlisted and compressed but are not age-encrypted individually; do not put titles, identifiers, credentials, paths, or private payloads in them.

The browser metadata export feature uses age directly: it fetches age ciphertext, decrypts in the browser with the user's private identity, and builds the export locally. The coordinator may relay the ciphertext and opaque filenames, but must not receive the identity or decrypted export. The export prototype is limited; see [metadata export plan](metadata-export-plan.md).

## In-transit browser-to-node content through the coordinator

TLS remains required for network transport, but TLS terminating at the coordinator does not provide end-to-end confidentiality from that coordinator. Sensitive application content therefore needs a second, application-layer encryption boundary whose endpoints are the browser and the authorized metadata or storage node.

### Current full Movies session design

The full coordinator Movies view uses a short-lived, per-request session encryption design:

1. The signed-in browser creates an ephemeral P-256 ECDH key pair using Web Crypto. The private key is non-exportable and remains in browser memory. The browser sends only the public key in the request body.
2. The coordinator authenticates and authorizes the user/node, selects a configured node target, and relays the request. It does not have the browser's ECDH private key.
3. The metadata node fetches its Radarr inventory, removes the local filesystem path and other unnecessary fields, serializes the browser view, derives a session key from ephemeral P-256 ECDH plus HKDF-SHA-256, and encrypts the response with AES-256-GCM and a fresh 96-bit nonce.
4. The metadata node returns its ephemeral public key, nonce, and authenticated ciphertext. The coordinator forwards these opaque bytes. It must not parse, transform, cache, index, or log the response body.
5. The browser derives the same AES key, authenticates/decrypts the response, validates the resulting inventory, and renders it locally. The age private identity is not used for this session.

The session key is fresh for each library load; the node's ephemeral private key and shared secret are discarded after the response. AES-GCM authenticates the ciphertext. Altered response bytes fail decryption. Transport-layer TLS still protects credentials, public-key exchange, response integrity to the browser, and network metadata from passive network observers.

### Important security qualification

In the current implementation, the coordinator relays the browser's public ECDH key to the node and returns the node's ephemeral public key alongside the ciphertext. The coordinator can replace either public key while relaying and establish two separate ECDH sessions, one with the browser and one with the node. Therefore this protocol protects against a passive or honest-but-curious coordinator that relays messages unchanged, but **it does not cryptographically prevent an actively malicious coordinator from performing a man-in-the-middle attack**.

Do not describe this exchange as fully end-to-end secure against an actively malicious coordinator until the browser can authenticate the metadata/storage node's public key independently of the coordinator. A future protocol should bind node identity and the ephemeral session key to a node-held signing key or another key pinned/provisioned out of band, include protocol and request context in the authenticated transcript, and define replay, expiry, and key-rotation behavior. Until then, the honest-relay assumption is part of the trust model.

### Coordinator relay restrictions and future node content

The coordinator's current node-page relay accepts only specific `GET` requests for the age-encrypted metadata export index, config, and validated ciphertext objects. General node pages and plaintext APIs are rejected by that relay path. The full Movies P-256/AES-GCM session uses a POST and is not included in the restricted export-only relay; as a result, the described new session is not yet reachable through the current coordinator relay allowlist. The full table and cryptographic endpoint exist in source, but their integrated coordinator flow is not yet deployed or end-to-end verified.

Do not expand coordinator proxying to arbitrary subscriber HTML/API responses. For every content type and node role, either:

- persist/publish age-encrypted objects and relay ciphertext plus opaque object references; or
- use an authenticated browser-to-node encrypted channel with independently authenticated node keys.

The same rule applies to storage-node metadata, artwork, library inventories, and future media types. A request body containing a public key can itself reveal which function is being called and its size; use generic opaque routes and avoid plaintext identifiers in URLs, headers, filenames, logs, and errors where practical.

## Network and transport security

- Use HTTPS for remote coordinator and node traffic. Localhost HTTP is a development exception. Do not expose the coordinator's PostgreSQL port publicly or through Tailscale Serve.
- On the MacBook deployment, Tailscale Serve HTTPS 443 routes to coordinator loopback `127.0.0.1:8080`. Tailscale is transport and routing, not authorization or content encryption.
- Metadata and storage app ports must never be published on the host. On OrbStack, the host can route directly to a container IP even when no host port is published. The MacBook metadata process shares the coordinator's Tailscale namespace and binds to `127.0.0.1:8081`, so the coordinator reaches it over shared loopback. The Mac mini storage process shares its Tailscale namespace and binds to `127.0.0.1:8080`; Tailscale Serve reaches it through an explicit HTTP proxy bound to `127.0.0.1:8082`. Verify host `localhost` and container-IP access both fail, while the coordinator relay and intended Tailscale route work.
- The coordinator relay uses fixed configured node targets and bounded request bodies. The ciphertext export path validates object names and only permits the intended GET endpoints. Keep relay allowlists narrow and fail closed on unknown methods/paths.
- Radarr/Sonarr upstream connections require HTTPS in production. Validate image source URLs and limit content types and sizes. Never forward upstream API keys to the coordinator or browser.
- Syncthing's GUI should stay loopback-only unless deliberately secured. Configure peers/folders only after owner approval; do not share `/media`, `/data`, node tokens, private identities, or databases.

## Storage and secret handling

- `/media` is local and read-only to Rawrarr on storage nodes. Never copy it into `/shared` or proxy it through the coordinator.
- `/shared` contains the public recipient and age ciphertext. Keep it outside the source checkout and synchronize only with explicitly approved devices.
- `/data` contains node-local state and possibly a development-only Radarr config mirror. Production disables the plaintext mirror. Restrict filesystem permissions and never sync `/data`.
- Keep PostgreSQL data and backup files local to the coordinator; protect database credentials with secret files and never print them.
- Do not commit `.env` files, tokens, private age identities, passkeys, API keys, database dumps, or runtime data. Keep secrets out of terminal output, tests, runbooks, screenshots, and support bundles.
- The browser stores the coordinator's session in an `HttpOnly` cookie. The age private ID is kept in origin-scoped `localStorage` so tabs in the same browser profile can unlock the library; it remains until the user clears it or signs out. Treat it as sensitive key material: never include it in coordinator requests. Same-origin scripts, browser extensions, injected scripts, compromised devices, and XSS can access it while stored or in use.

## Logging, errors, and observability

Request logs should record method, route, status, and duration only. Do not log query strings, authorization headers, cookies, request/response bodies, age identities, ECDH private keys, session keys, titles, TMDb IDs, media paths, API keys, or decrypted configuration. Errors returned through the coordinator should be generic and should not include upstream payloads or secrets. Health endpoints report process responsiveness only; they are not evidence of node trust or end-to-end security.

## Threats this design does not eliminate

- A compromised browser, browser extension, endpoint, metadata/storage node, or its administrator can read plaintext available at that endpoint.
- A compromised or malicious metadata/storage node can return false content, retain plaintext, or misuse its upstream credentials. The browser currently authenticates the coordinator session and node ownership, but the new ECDH exchange does not yet independently authenticate the node's ephemeral key.
- The coordinator observes routing, node/account associations, timing, sizes, and presence. Under the current ECDH exchange it can actively substitute keys and read/modify the session if malicious.
- TLS/Serve, Tailscale, Syncthing, passkeys, node approval, and age solve different problems. None replaces the others.
- Backups, screenshots, OS swap/crash dumps, browser devtools, local plaintext development config, and user-created exports can expose data outside the encrypted shared-file format.
- Availability attacks, traffic analysis, endpoint compromise, denial of service, and malicious software updates remain possible.

## Security verification expectations

For changes that affect sensitive data flows, add focused tests that demonstrate:

- known title, provider-ID, path, and artwork markers are absent from coordinator request/response bodies, database records, and logs;
- authorized browser decryption reproduces expected content, while modified ciphertext, wrong keys, malformed keys, and oversized bodies fail closed;
- node ownership, role, session, and allowlist checks reject unauthorized requests;
- private age identities, node tokens, API credentials, and private session keys never enter coordinator requests or persistent shared data;
- TLS routes to the coordinator, and Tailscale Serve status matches the intended loopback upstream;
- migration or deployment changes preserve existing volumes and secret mounts.

## Current implementation status

The age-encrypted config/metadata mechanisms, post-quantum age identity handling, passkey auth, node approval, HMAC metadata filenames, bounded ciphertext export relay, and browser-side age export/decryption code are implemented. The new full Movies view and browser-generated P-256/AES-GCM response code are present in source, but the POST flow is not yet admitted by the coordinator's export-only relay, its node key is not independently authenticated, and live end-to-end decryption is unverified. The current MacBook metadata and Mac mini storage runtimes share their Tailscale namespaces and bind to loopback, as recorded in [the host-isolation runbook](../runbooks/2026-10-18-isolate-node-containers-from-host.md). Treat the intended blind-relay property as incomplete until those unrelated relay gaps are closed and validated.
