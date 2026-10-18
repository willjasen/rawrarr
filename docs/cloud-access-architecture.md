# Rawrarr cloud access architecture

## Product model

Use a cloud portal similar in purpose to the user's UniFi analogy: a person visits a Rawrarr domain, signs in to an access portal, and reaches a node they own or have been invited to use. The portal helps identify the instance and find an online device. The node remains the controller and authority for its configuration, media, users, and permissions.

This is an architectural analogy, not a claim about UniFi's internal implementation. The Rawrarr design goal is that the cloud can provide account, discovery, subscription, and connectivity services without receiving the contents of a subscriber's node.

## Components and trust boundaries

The operator-owned deployment is the **service node**; subscriber deployments use **coordinator**, **storage**, and **shard** roles, with coordinator and storage optionally combined. The service node will eventually run SwarmSail to provision one isolated Headscale tailnet per subscriber account. Tailscale is an overlay transport only: it does not grant access to shared Rawrarr data, which remains controlled by Rawrarr permissions and age recipients. The service node remains outside each subscriber's shared folder and is not a Rawrarr app backend. See [node types](node-types.md).

| Component | Responsibility | Data it may know | Data it must not access |
| --- | --- | --- | --- |
| Cloud portal | Sign in to the Rawrarr access service, list authorized instances/devices, start a connection | Portal account, opaque instance/device IDs, access grants, online status | Media files, age private identity, plaintext config, library contents, node database |
| Rendezvous/relay | Find an online node and forward encrypted connection bytes when direct access fails | Opaque IDs, transient route, source/destination network metadata, byte counts | TLS plaintext, app content, media or config payloads |
| Rawrarr node | Serve the actual app and enforce authorization locally | Its local config, local database, shared app data it can decrypt, local `/media` if mounted | Another node's local media path unless a separately authorized device operation accesses it |
| Licensing service | Verify billing events and issue signed subscription entitlement | Billing provider reference, opaque instance ID, plan and status | Node contents, config, age identity, media inventory |
| Syncthing peers | Replicate the shared Rawrarr app-data folder | Folder members, filenames, file sizes, ciphertext | Age private identity and plaintext of correctly encrypted records |

The access portal and licensing service have different jobs. Portal sign-in grants permission to attempt access; Rawrarr on the destination node still authenticates the user and enforces the current instance policy. A portal account is not an age identity or a Rawrarr node credential.

### Relay and subscription server data boundary

The cloud relay/subscription service may keep its own passkey account records and associate one or more **public age recipients** with the matching passkey-authenticated account in PostgreSQL. A public recipient is public key material: it can be used to encrypt data for its owner, but it cannot decrypt that data and does not authenticate the owner. The relay may use it for device discovery or recipient lookup. Do not store a private age identity there.

The planned account schema should make this relationship explicit: account rows own passkey credential rows and public-recipient rows, with a join table associating each recipient with the account's passkey credential(s). This lets a user manage multiple passkeys and public recipients without treating either value as a substitute for authentication. This schema belongs to the cloud account service; it is not the subscriber node's PostgreSQL database.

Subscriber app data remains on subscriber-controlled nodes and in their Syncthing folder. The relay must neither encrypt nor decrypt subscriber app data, and must not persist its payloads. For remote access it forwards the end-to-end encrypted TLS byte stream unchanged; TLS and application decryption terminate at the user's node/client. Keep billing and route metadata separate from payload data. The relay's own account, passkey, public-recipient, routing, and subscription records are service data, not a copy of the subscriber's Rawrarr app data.

## Connection flow

1. A node initiates an outbound authenticated connection to the Rawrarr rendezvous service, so it can be reached behind NAT without opening an inbound port.
2. The node registers its opaque `instance_id` and `device_id`, supported app endpoint, and short-lived online state. It does not upload its config or media inventory for routing.
3. The user signs in at the Rawrarr cloud portal and selects an authorized instance/device. The portal returns a short-lived, audience-bound connection ticket.
4. The browser attempts a direct peer connection where available. If that fails, an opaque relay forwards the encrypted stream between the browser and node.
5. TLS and Rawrarr application authentication terminate at the node. The browser receives the app directly from that node over the end-to-end session; the portal must not inject executable UI code into a session that promises relay-blind privacy.
6. The node independently checks passkey/session state, instance membership, and requested action. A route ticket or knowledge of an ID never grants content access.

## Docker deployment and node claim (planned)

1. After the owner chooses private or subscribed deployment and a node role, the portal displays a role-specific Docker Compose YAML file for the owner to save and start with `docker compose up -d`.
2. The container creates a device key locally and makes an outbound TLS connection to the operator's service node. It registers only its public device identity, selected role/capabilities, and opaque pending enrollment ID; it does not upload `/shared`, `/media`, its local database, or an age identity.
3. The authenticated owner signs in to the portal with a passkey, reviews the pending node, and explicitly claims it into a private workspace or subscribed account.
4. The node verifies the claim response against its locally generated key, stores its enrolled identity locally, and begins its authorized account-level overlay connection if enabled. Portal claim state does not grant access to shared Rawrarr files or override node-local authorization.

Compose output must not embed a reusable enrollment token, passkey, or age identity. A startup log may point to the portal using a non-secret URL or pending-device identifier, but not contain a bearer claim secret. If bootstrap needs a transient proof, it must be single-use, short-lived, audience-bound, and protected from Compose files, logs, shared folders, and service database persistence. The portal should state what metadata enrollment sends upstream. Private-mode operation and subscriber entitlements are separate modes; subscription expiry must not delete the user's node data.

This is planned behavior. Device registration, claim lifecycle, generated Compose files, key continuity proofs, and service-side entitlement enforcement are not implemented yet.

The connection portal may know who has an account and which IDs they may attempt to reach. It must not have a server-side admin channel into a node. Recovery or support access, if ever offered, needs an explicit separate user grant and cannot silently bypass local authorization.

## Relationship to other transports

- **Tailscale:** optional private transport for Rawrarr web/API, control, and app-data traffic. It is not a media-file or media-stream transport.
- **Syncthing:** copies shared Rawrarr app data between selected peers. It is not an online quorum, a database replication protocol, or a media-transfer feature in this scope.
- **Rawrarr cloud relay:** future access path for the app session. It forwards encrypted bytes when needed and does not replace Syncthing or manage media storage.
- **Storage nodes:** keep media local under `/media`; Rawrarr may send a narrow scan/control request to a node, but the cloud access service never proxies media bytes.

## Design constraints

- Keep the subscriber's Rawrarr instance and nodes self-hosted; the cloud portal is not the application backend.
- Never transmit a private age identity to the relay/subscription server or place it in relay storage, PostgreSQL, logs, telemetry, environment variables, or process memory. A user's own Rawrarr node is a separate trust boundary.
- Permit public age recipients in the relay account database, explicitly linked to the passkey-authenticated account; never treat a recipient as a login credential.
- Do not place subscriber app data, plaintext shared configuration, node PostgreSQL files, media paths, or media bytes in portal or relay storage.
- The relay must not age-encrypt or age-decrypt subscriber app data. It forwards end-to-end encrypted traffic and does not keep payload copies.
- Use opaque IDs as locators only, never as credentials. Authenticate device registration and issue short-lived route tickets.
- Prefer direct connections and use the relay only when direct connectivity fails.
- Minimize and expire route logs and online-presence data. Never log payloads or credentials.
- Preserve local access when the portal, relay, Tailscale, or Syncthing is unavailable, subject to the existing local app behavior.
- Do not conflate cloud portal authentication, node-local passkey login, age decryption, and subscription entitlement.

## Decisions and implementation sequence

1. Decide whether the cloud portal uses a separate account or federates an existing identity, and how its account maps to instance invitations.
2. Define portal roles and revocation semantics, including what happens when an invitee is untrusted or a shared encrypted object cannot be revoked.
3. Define node registration, device-key enrollment, route-ticket scope/lifetime, and device revocation.
4. Select direct-connect and relay protocols, end-to-end TLS/certificate issuance, NAT traversal, and abuse controls.
5. Prove portal and relay blindness with a threat model and packet-level tests; verify that the node is the TLS/application endpoint and relayed bytes are unchanged.
6. Keep subscription entitlement verification independent, with only the opaque instance ID and license state crossing that boundary.
7. Test portal/relay outages, nodes behind NAT, direct-connect fallback, expired/replayed tickets, revoked devices, and node-local authorization. Verify passkey-account to public-recipient association and prove no private identity or app-data plaintext enters relay databases, logs, memory, or payload handling.

This plan does not select a cloud vendor, tunnel protocol, identity provider, or relay implementation. It records the trust model before implementation.

## Related architecture

- [Device architecture](device-architecture.md) — node roles, shared instance identity, pairing, and relay boundaries.
- [Optional Tailscale plan](tailscale-integration-plan.md) — private Rawrarr app connectivity.
- [Subscription model](subscription-model.md) — hosted checkout and signed entitlements.
