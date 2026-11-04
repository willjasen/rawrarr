# Rawrarr product and interface design

> Updated container architecture (2026-10-18): central **coordinator** owns passkeys/authentication and node coordination; **storage** and **metadata** serve the app, Radarr, and shared data. Only storage reads media. Metadata replaces shard; the MacBook app is metadata. See [implemented deployment and Mac mini setup](container-deployment.md), which supersedes older role/startup statements below. Manual node approval is implemented; browser device-code linking remains planned.

This is the living product-design guide for how Rawrarr should behave and feel to its users. It captures user-facing decisions and expectations, rather than implementation details. For technical boundaries and mechanisms, see [data domains](data-domains.md), [device architecture](device-architecture.md), and [technical status](technical-status.md). For how to use the current prototype, see the [user guide](user-guide.md).

## Design goals

- **Private by default.** Make it clear which information is public, encrypted, shared, or local. Do not reveal private identities, credentials, media paths, or full identifiers in ordinary UI.
- **Friendly and calm.** Use the Rawrarr dinosaur identity and green accents with a clean, modern interface. Keep setup focused and avoid overwhelming first-time users with advanced configuration.
- **Show useful status.** Tell the user whether an integration or device is connected, when information last changed, and what action can fix a problem. Prefer live updates over asking the user to reload.
- **Keep control with the user.** Make sharing, scanning, backup, and connection actions explicit. Explain the effect before an action that changes or sends data.
- **Remain understandable when nodes are offline.** Show which data is local or shared, which device can perform an action, and whether it is waiting for a peer to reconnect.
- **Design for “eventually.”** Distinguish app configuration data from app metadata. Radarr settings (configuration data) follow the authenticated RPC/Merkle work; age-encrypted app metadata such as movie records and artwork lives under `/shared/in/metadata` and replicates with Syncthing. Current code copies missing legacy metadata files from `/shared/metadata` on vault unlock without deleting or overwriting. `/shared/out` is for larger content deliberately published to peers, such as selected media collections. Metadata should converge when approved peers reconnect; databases and other node-local state remain local unless explicitly designed otherwise. Sharing media requires deliberate owner approval; Syncthing transport does not encrypt files at rest.
- **Support sharing beyond media.** Treat device-to-device file sharing as a core direction for Rawrarr, following the broader Synkpieces idea. Users should eventually be able to share ordinary files and folders; media automation is the first focused use case, not a required data type.

### Deployment definition organization

Keep generated subscriber installations, operator deployments, and isolated tests
distinct. Compose cleanup proceeds in caller-traced groups, preserving explicit
project/resource names and data-domain boundaries rather than hiding incompatible
topologies behind one universal stack. The first implemented group lives under
`deploy/testing/`; the former Mini coordinator and empty Serve definitions are
retired, with current regression coverage pointed at do3. The MacBook coordinator
overlay and obsolete staged Headscale clients are retired too; the admin preview
is a standalone localhost-only container. The internal coordinator stays on do3,
not either Mac. Other role-base
consolidation remains planned under
[#154](https://github.com/willjasen/rawrarr/issues/154). See the
[Compose inventory and adoption guidance](compose-layout.md).

The old additional MacBook boneyard and Mini-specific transport/Caddy relay
templates are also retired with explicit operator confirmation. Current
host-local fossil deployments and generic role templates remain separate;
source retirement never removes live identity state, data, or routes.

## Visual identity

Rawrarr should feel approachable and distinctive without making the interface look like a toy. Use a restrained dark or light surface system, readable typography, clear section hierarchy, and green as the primary brand accent. Reserve bright colors for status, identity illustrations, and meaningful actions.

Use **“all the arrs”** as the Rawrarr tagline beneath the product name in the public landing-page header.

Serve the brand logo as a lossless JPEG XL image where the browser supports it, with a PNG fallback. Future logo animation may be used sparingly for selected moments; keep a static fallback and respect reduced-motion preferences.

Always render the product name as lowercase **rawrarr** in user-facing interface text, errors, and logs.

A generated identity illustration and the active workspace color scheme should be derived deterministically from the public age recipient. The same recipient should always produce the same logo and palette across nodes. Keep the normal Rawrarr brand palette before an identity is created or unlocked; switch to the recipient-derived palette after creation or successful import. Use a recognizable multicolor geometric identity mark and a coordinated accent/surface tint in the app. Never place the full recipient or private identity inside the illustration or its accessible label. The logo and palette identify an age identity visually, while the displayed short form lets a person distinguish nearby identities. Preserve contrast and semantic status colors so color alone never communicates status.

The signed-in profile picture uses a SHA-256 digest of the public age recipient as its deterministic seed. The browser reads the recipient from an authorized fossil node through the existing coordinator relay, computes the digest, and requests a JPEG XL from the authenticated coordinator endpoint, with a PNG fallback for browsers without JPEG XL decoding. The coordinator generates the SVG internally, rasterizes and encodes it, then returns only the raster image; SVG markup is not sent to the browser, and no image or avatar seed is stored in PostgreSQL. Different recipients produce different dinosaur types, colors, crests, spots, zero to three back spikes when wingless, tail lengths, optional wings, rare golden palettes, and volcanic, forest, hilly, desert, snowy, or tropical background scenes. Forest trees are placed to either side of the dinosaur so their trunks and foliage remain visible.

Treat avatar artwork as versioned generations over one stable identity seed. A renderer generation describes its composition, drawing method, and output quality; it may improve shape design, detail, shading, and raster resolution while preserving the same account identity and stable high-level traits where practical. Keep earlier generations available to a comparison preview so the owner can judge visual progress side by side. Do not silently change a person's identity seed to make art look different, and do not imply that generations evolve automatically over time: a new generation is an intentional design update. The active app may use the newest approved generation while comparison tooling can render any retained generation against fixed seeds.

## Account and first-run flow

Before sign-in, the coordinator root route shows a public landing page that introduces Rawrarr, its current Radarr/Sonarr and device roles, and the passkey sign-in path. Keep the page useful without an account, responsive, and explicit that Rawrarr is under active development. Explain privacy goals alongside the coordinator's visible operational metadata and the current limits of unverified content flows. Do not present design goals as a completed security audit or imply that future general file-sharing and subscription features are already available. An optional public demo may offer a guided, read-only tour with isolated synthetic data; it must not grant authenticated access or touch subscriber data. See the [public demo plan](public-demo-plan.md).

The operator admin console is a separate, private service, not a coordinator page or public product route. Its first preview uses unmistakably labeled sample data and has no connection to production accounts. Localhost and Tailscale reachability are network boundaries, not a substitute for the planned administrator login; require server-enforced admin authorization before showing live user data. See the [admin console architecture](admin-console.md).

1. Check PostgreSQL for an existing passkey account. If one exists, show only **Sign in with passkey** and use the stored account identity. If none exists, then ask for the user's name and offer **Create passkey**; derive the internal username from that name.
2. After a first passkey is registered, take the user to the age setup screen before the main app.
3. Let the user create a new age identity or paste an existing one into a masked field. Explain that the identity protects shared configuration and metadata. Keep copy confirmation distinct from unlocking the encrypted vault.
4. Show only shortened identity forms. Never render the complete private identity or complete public recipient in page text, labels, tooltips, or logs.
5. After the user selects **Create ID**, open a pop-up that guides them through copying only the private key line. Show **I copied it and saved the private ID somewhere safe.** only after clipboard copy succeeds. Keep **Continue** visible but disabled until copy and acknowledgement succeed, then require an explicit click to unlock and open the workspace. Keep the recovery warning yellow and show progress/errors inside the dialog. Do not provide a Close action. Show only the first 12 and last 5 characters of the generated identity with an ellipsis. The three-character check is temporarily disabled; restoring it is tracked in [ROADMAP](../ROADMAP.md). The acknowledgement does not prove where the user saved the key.
6. Show the recipient-derived colored logo and only the first 12 and last 8 characters of the public recipient with an ellipsis. Never render the full recipient in page text, labels, tooltips, or logs. Keep the full private identity out of rendered page text. Use it for copying and deliberate unlock; persist it only on the subscriber node when the user explicitly selects the local remember option.
7. Put existing post-quantum identity import behind an **Import ID** action that opens a focused pop-up with a masked paste field and explicit unlock action. Do not provide a file-selection action. Keep the copy pop-up and backup acknowledgement for newly generated identities.
8. After the vault is unlocked, take the user to the main workspace. On a later session, require passkey sign-in and age-vault unlock again unless the user explicitly chooses **Keep this ID on this device**. Store that opt-in identity only in node-local app data with owner-only permissions; never put it in `/shared` or sync it.

The shortened display is an explicit privacy decision: partial forms are for recognition; they are not a substitute for the complete identity and must not be expandable inline.

### Environment ownership and coordinator backups

An environment owner is an explicitly authenticated existing Rawrarr account,
not an age group or device identity. A deliberate owner API action selects an
owned online fossil and obtains only the public recipient derived from its
existing user age identity. PostgreSQL pins that recipient with the account and
source provenance independently of ML-DSA signing-root material. Identical retries
are safe; conflicting keys or owners are rejected, never silently replaced.
Subsequent internal coordinator backups use the public pin even when fossils are
offline. The private identity remains on the fossil or owner-controlled recovery
device. This does not add a settings UI or automatic key rotation. See the
[owner-recipient runbook](../runbooks/2026-11-03-environment-owner-backup-recipient.md)
for adoption prerequisites and the distinction between implemented code and
deployed behavior.

### Account tiers, ENS sign-in, and voluntary ETH

Public accounts have three access states: a three-day trial, a permanent free tier limited to one fossil node, and a paid subscription tier. The trial begins after account setup, as it does today. After the trial ends, the account falls back to free unless a verified subscription is active. Sign-in, account recovery, data export, and deletion remain available at every tier, and a tier change never deletes subscriber data. The coordinator must enforce node limits on the server when accepting enrollment and again when approving a node; hiding an action in the browser is not enforcement. Internal deployments remain outside public trial and payment behavior.

Anyone may send a voluntary ETH donation without signing in. A donation does not create an account, extend a trial, change a tier, or unlock features. Donations and subscription payments are separate flows and must be labeled separately in the UI.

Wallet sign-in is an optional account authenticator alongside passkeys. An owner first signs in with their existing passkey and links a wallet by entering an ENS name such as `willjasen.eth`. The coordinator resolves it on Ethereum mainnet, asks the wallet controlling the resolved address to sign a Sign-In with Ethereum (SIWE) challenge for the current Rawrarr origin, and verifies the signature, one-time nonce, chain ID, expiration, and current ENS forward resolution before linking it. Later, the owner may enter that ENS name and sign a fresh challenge to receive the normal coordinator session. The ENS name or its records alone are never credentials. Rawrarr never asks for or handles a seed phrase or private key.

The first subscription flow is a user-initiated ETH transfer on Ethereum mainnet. Automatic recurring charges require a future, separately reviewed and audited smart contract. Subscription UI must remain disabled until an operator configures the treasury address, plan amount, and period. Only a successful direct ETH transaction from the account's linked wallet to that address, for the exact configured amount, and included in Ethereum's finalized canonical chain may extend paid access. A transaction hash by itself, a receipt from another network, an internal contract transfer, an unconfirmed transaction, a donation, or an underpayment never grants access. Manual renewal means that each period requires another explicit wallet transaction.

Do not expose payment-provider or RPC credentials to the browser, subscriber nodes, `/shared`, or local media paths. The coordinator needs only public wallet addresses, ENS names, transaction hashes, and verified payment metadata. Never store private wallet keys or seed phrases. Use the current shared-instance/account entitlement boundary consistently with the billing design; subscriptions do not bill by node count. Price, billing period, paid-tier node limits, and support for non-fossil roles on the free tier remain product decisions and must be configured or settled before live rollout.

### Planned one-line node deployment

Extend the subscriber Compose preview with **Copy install command** and **Download/review script**, while retaining manual Compose download. The copied curl-based command runs on the destination node and downloads a script containing its generated Compose definition. Explain Docker prerequisites, platform limitations, requested host changes, and local storage before execution; installing Docker requires explicit consent. Keep media-path entry and private material on the subscriber side.

Show **Started; waiting for owner approval** separately from a usable, approved node. Installation must not approve enrollment, unlock an age identity, or configure sharing automatically. Preserve `127.0.0.1:7297`, safe reruns, existing data, and other Docker workloads; stop on conflicts instead of silently changing ports. This flow is planned and depends on published images and a tested download/bootstrap path. See the [one-line node deployment plan](one-line-node-deployment-plan.md).

## Main workspace

The main workspace should make the user's shared files, media, and service health easy to scan at a glance. The first version can stay focused on the media library, but the workspace should not imply that all shared data must be a movie, episode, album, or book. Keep the primary areas recognizable:

- **Library:** files and folders shared through Rawrarr, with media views for movies and other recognized media. Show clear file, metadata, and availability states without requiring every item to have media metadata.
- **Activity:** recent scans, metadata updates, service events, and actionable errors.
- **Devices:** nodes and their roles, connectivity, shared-folder status, and the data each device can access.
- **Settings:** connected Arr services, metadata sources, and app preferences.

Use plain language in empty states. When there is no library content, direct the user to configure a media folder and scan it. When an integration is disconnected, show its last known state and a specific next step.

## AI chat and request review

The first implementation supports signed-in, read-only movie year and genre questions and compares the reported disk usage of Movies and TV in roarlay. OpenAI interprets the question, while the browser filters its decrypted movie records and calculates percentages from Radarr and Sonarr sizes. For text chat, roarlay streams a brief OpenAI introduction through SSE; the browser then appends its grounded library answer. The streamed request contains the question but no decrypted catalog. The assistant page has separate Chat and rawrs voice tabs and a standing notice that words or recordings go to OpenAI. Press-to-talk sends a short recording to OpenAI for transcription, then uses the same question path; a local browser voice speaks the full answer when available. Roarlay stores questions, channels, status, and interpreted filters in local PostgreSQL. The daily five-question limit is the default; an owner-account-only local environment override temporarily lifts it during testing. Trial and paid entitlement records are not yet implemented. A missing OpenAI key disables the interface. Voice is turn based and requires a microphone-capable browser.

The current chat is a narrow intent classifier and local dispatcher, not a general agent. When it cannot map a question to one of its supported intents, it marks the request for review and shows this response:

> I will check the available library records for this question.
>
> I saved that question for review. I can search your movies by year or genre right now. What would you like to try?

Replace this dead end with OpenAI Responses function calling and an application-owned registry of narrowly scoped tools. Begin with read-only tools for movie search and library statistics; validate every argument and execute only through Rawrarr's authorized application paths. Keep decrypted catalog filtering in the browser and return results directly to the user by default. If a future flow asks the model to explain private results, disclose exactly which result fields would be sent to OpenAI and require the corresponding user setting or consent. Mutations such as configuration changes, downloads, or sharing edits must create a proposal for explicit owner approval, never execute solely because the model requested a tool call. Do not give a model direct PostgreSQL, shell, or unrestricted configuration access.

An MCP server is not required for the built-in Rawrarr chat. Responses function calling can let the model request one of Rawrarr's own application functions, which roarlay validates and executes. Once these scoped capabilities are stable, an MCP adapter may expose selected tools to a separate owner-run review agent or other compatible clients. Keep it as an adapter over the same authorization and operation layer, not a direct database tunnel; a remote OpenAI MCP connection would also need a reachable, authenticated endpoint or a supported private tunnel. Select a currently supported model during implementation: the currently configured GPT-4o mini is documented as supporting function calling, but is now marked deprecated.

The owner review agent, configuration proposals and approvals, retention controls, searchable inbox, Twilio entry point, and entitlement-specific quota remain planned.

Give the text and voice assistant an innocent, warm, curious personality: a tiny, thoughtful dino helper who is helpful, patient, gently upbeat, and a little goofy. It can ask a short follow-up when a question is unclear and sound pleased to help without becoming sugary. Keep its playfulness occasional and its answers easy to understand. It must stay candid about missing records and never claim to have found titles it has not checked. The character is a communication style, not a claim to be a human child or to have personal emotions or experiences.

Offer an optional chat entry point in roarlay for questions about the owner's library and for narrowly scoped actions. For example, “Which of my movies were made in the 90s?” should resolve to a movie release-year filter (1990 through 1999), return matching titles from actual library records, and show the source and freshness of those records. If the library is unavailable or a field is missing, say so rather than inventing a result. Start with read-only library questions, then consider metadata refresh requests, recommendations based on owned titles, and draft requests for missing movies or shows.

Record each submitted question in coordinator-local PostgreSQL as an owner-reviewable request, with its requester, time, interpreted intent and filters, result or failure state, and any proposed follow-up. Make the history searchable and deletable with a defined retention setting. Treat the question text as private user data. A separately authorized, owner-run agent may read the request inbox and propose metadata refreshes, catalog improvements, or configuration changes; show what it proposes and what actually happened. A question does not by itself authorize a configuration edit, download, sharing change, or other mutation. Require an explicit owner action for such changes, and retain an audit trail of approval and outcome.

Use existing metadata to answer a question before proposing a configuration change. A missing release year calls for a metadata refresh or catalog repair; adding a new source or changing an integration belongs in age-encrypted configuration only if the owner approves it. The agent must use scoped application operations rather than arbitrary SQL or shell commands. Cloud AI receives only the question and minimal context needed for its task, with an explicit owner-controlled enablement and clear notice of what leaves the instance. Keep private age identities, credentials, media bytes, and plaintext shared configuration out of cloud requests and the operator service. Roarlay must preserve its blind-content boundary: library records are filtered through an authorized metadata path or the owner's browser, and any future cloud access to decrypted titles requires a separate, explicit product decision.

Add a talking voice mode for this same assistant. A signed-in owner can start a microphone conversation from roarlay, ask the same library questions as in text chat, hear the answer, and see a transcript and request status in the workspace. A later optional Twilio phone entry point may support callers away from the browser. Link every voice request to the same PostgreSQL review inbox with its channel and call/session identifier, transcript, interpreted intent, answer, and follow-up state. Explain when speech leaves the device and whether a transcript is retained; do not record or retain raw call audio by default. A phone number or caller ID alone does not authorize library access or mutations: require an owner-linked authentication step before revealing private results, and keep unauthenticated callers to generic help. The voice agent uses the same scoped operations and owner-approval rules as text chat, and must confirm a request aloud before submitting it.

During an active trial, allow each signed-in account up to five AI questions per UTC calendar day, shared across typed chat and voice. Count one accepted user question or voice request, not internal model turns, tool calls, or the owner's separate review-agent work. A failed request that produces no usable answer or accepted follow-up should not consume a use; use a stable request ID so retries cannot count twice. Show uses remaining and the next reset time in the user's local time before submission and when the limit is reached. Enforce the limit on roarlay against coordinator-local PostgreSQL, including authenticated phone requests. Trial expiry still takes precedence over any unused daily allowance. Paid-plan limits remain a separate decision.

## Integrations and library behavior

### Arr summary cards

Render the four-card summary row for each Arr library through one shared component. Keep each service's card labels, details, and value keys in a service definition; the renderer owns the common markup, layout, value updates, and reset behavior. Radarr and Sonarr use the same concepts where they match, while their labels and explanations name the service and its media type. A future Lidarr, Readarr, or other Arr page should add a service definition and reuse the renderer instead of copying card markup or formatting logic. Only include a metric when that service can report a value with the same meaning.

The metadata-size card reports the selected service's media subtree (Radarr movies or Sonarr TV) and shows the complete shared metadata tree as a secondary “All metadata” value. Both sizes count stored encrypted files and use binary units. The service subtotal must not be described as all Movies-and-TV metadata. Hide a backfill completion message when every eligible artwork item is saved, even if the worker remains active checking for new library entries.

### Radarr

Treat Radarr as the first supported external library integration. Let the user enter its URL and API key in the UI, validate the connection, and apply saved settings without requiring an app restart. Show connection status and movie data in the workspace. Mask or redact the API key everywhere it is displayed after entry.

### Radarr recovery and node migration

Keep the latest native Radarr backup in the shared Syncthing folder as age ciphertext, refreshing it every six hours when Radarr is connected and the age vault is unlocked. Rawrarr's scraped movie metadata and artwork live under `/shared/in/metadata` alongside the native backup. Together these support disaster recovery and migration, but Rawrarr does not create or control the Radarr container: the user deploys it on a chosen metadata or storage node and initiates restore there. Restore Radarr's database and settings without copying or modifying media; preserve the backup's root-folder paths and require matching mounts or user repair after restart. Keep Sonarr out of this first implementation.

When the Radarr UI requires authentication for its backup ZIP, accept optional UI credentials alongside the API key. Store them only in the age-encrypted shared config, never in the node-local plaintext mirror or settings responses. The worker must keep redirects disabled, send credentials only to the configured Radarr HTTPS origin, and never place them in a URL. A blank credential pair preserves the existing encrypted login; clearing the Radarr connection removes it.

### Direct Radarr UI access (planned)

Let the user open the Radarr UI managed by a selected subscriber node, while the browser communicates with that node directly. The coordinator must not proxy, receive, or log the Radarr UI session. Reuse the established node-direct route where possible; do not expose Radarr through a coordinator reverse proxy.

### Local scans and metadata

Let the user choose a mounted media location and start a scan deliberately. Explain that a scan reads media and creates Rawrarr metadata; it does not move or rename media files. Show scan progress and per-item matching status. When a match is uncertain, present choices rather than silently attaching incorrect metadata.

Metadata and artwork that Rawrarr shares should be identified as encrypted. Explain when the app cannot create an encrypted sidecar because the shared public recipient is unavailable. Keep media files themselves distinct from metadata and app data in every screen.

For Radarr-backed movies, load shared encrypted poster artwork first. Scrape and save the full text record Radarr exposes, including ratings, genres, studio, language, status, quality profile, overview, and available movie-file details. Omit node-local paths and byte-size observations; boneyard nodes publish those separately under their own library sidecar domain. Store each provider record as age-encrypted JSON with an HMAC-derived filename in the shared text metadata tree, separately from artwork. Apply the same model to Sonarr series and episode records. Continue to show live artwork if shared storage is unavailable and run bounded sequential backfills.

### Future discovery and fulfillment

Rawrarr is intended to grow beyond Radarr into discovery, library management, and fulfillment. Present these as capabilities only when they exist. Keep provider-neutral concepts understandable to users and make the selected search or transfer provider visible when that affects a request. Do not imply that Rawrarr currently downloads or imports content through a concrete client when it does not.

## Devices and sharing

### Isolated and shared cluster modes (exploratory)

> This section contains multi-person cluster design proposals, not implemented behavior. Its per-file recipient-encryption discussion is exploratory and is not the agreed group/app key lifecycle. Use [identity domains](identity-domains.md) for the current agreed identity model and the implementation status of each key domain.

Create each new cluster in **isolated** mode, with one owner. The owner may make a one-way transition to **shared** mode when inviting another person. Once shared, that cluster identity must never return to isolated mode: removing every invitee or revoking every grant does not erase the fact that multiple principals may have seen data or keys. Show the transition and its permanent consequence before the owner confirms it, and record the mode in authenticated, rollback-resistant cluster state. A user who later wants an isolated cluster creates a new cluster with a new age identity and signing root; this is a new security domain, not a rollback or proof that the former member cannot access data they already received.

The cluster needs a verifiable history from its creation onward, including while isolated and as the owner adds or removes their own devices. Anchor that history in an owner-pinned genesis record and link each authorized change to the preceding signed record. Protect copies across owner-controlled devices and backups, detect rollback or forks, and make any trust-root rotation visible in the chain. If all history copies and valid recovery authorities are lost, Rawrarr cannot safely recreate the original trust history; it must treat recovery as a new trust root or use an explicitly provisioned recovery authority.

Sharing must not mean copying the owner's passkey, age private identity, or root signing private key. Passkeys authenticate individual people and remain in each person's authenticator. Each person should have their own age identity. For each file, age generates a fresh random file key, encrypts the payload once, and writes a recipient stanza for each authorized public age recipient; each stanza lets that recipient recover the same file key. The exact stanza contents depend on the age recipient type and do not necessarily include the public recipient ID verbatim. Each extra recipient adds a small amount to the file header; the encrypted payload is not duplicated. Rawrarr should keep public recipient IDs in its authenticated membership record to select recipients for new files. The shared folder contains the encrypted file and encrypted stanzas, never a plaintext file key or persistent cluster-wide pre-shared key. Rawrarr does not need a separate AES file-encryption layer or cluster key to meet the encrypt-once requirement. Existing age ciphertext encrypted to one recipient would need its header/key wraps updated to add another recipient; the payload need not be decrypted and re-encrypted, but Rawrarr would need a safe, supported rewrap implementation rather than assuming the age CLI provides this operation. Removing a recipient cannot undo access to a file, key, or plaintext already obtained. New files after removal should target only current members. A user can move ciphertext into a new isolated cluster without re-encrypting the payload only if recipient access is safely rewrapped and the former member never obtained a usable key; otherwise old content remains potentially accessible to them.

Keep signing keys separate from age encryption recipients. An ML-DSA or other signing certificate proves authorization or integrity; it is not an age encryption key and should not be converted into one. A signature can authenticate a membership or recipient change, while age recipients determine who can decrypt.

Cluster membership is owner-controlled. The owner may invite or remove members; an invited member may use the cluster but cannot invite another person or approve a new member. If MLS is selected for evolving group keys, this is an application authorization rule layered on MLS: every device must validate that membership-changing commits are authorized by the owner before applying them. Use one owner-authorized commit stream per cluster so the Syncthing-backed eventual delivery path has a single canonical membership-change writer. Non-owner proposals must not be committed, even if they are valid MLS messages. Define owner succession and recovery separately; do not silently promote an invitee.

Revoking a member must stop future access but cannot claw back data already received. Request that the member's Rawrarr nodes delete Rawrarr-managed shared metadata and cache data, but never their original local media. Show access as revoked separately from cleanup: pending while a node is offline, node-reported complete after an authenticated acknowledgement, or unconfirmed when no reliable acknowledgement arrives. Require a reconnecting revoked node to process the revocation and clean up before it rejoins current synchronization. Never describe an acknowledgement as proof of secure erasure; a person controlling the node can retain copies or backups.

The cluster root signing key is a distinct authority and remains private to its creator, ideally offline or in protected owner-controlled storage. It must never be shared as an invitation credential. Shared mode needs a defined multi-owner governance model: member-scoped signing keys and signed grants under the root, with explicit approval and revocation rules. Root-key loss/recovery, owner succession, and recovery from a compromised root must be designed before implementation; do not imply a passkey or age identity can recover it. Until those rules exist, shared mode is a product direction, not an implemented capability.

Use **coordinator node** for the primary UI/API, passkeys, PostgreSQL, account
ownership, catalog, and coordination; **fossil node** (`metadata`) for Arr and shared
metadata services; and **boneyard node** (`storage`) for read-only local media
access without user passkeys or PostgreSQL. A **dino** is a user-deployed node:
it may run fossil and either boneyard or the planned **tarpit** encrypted-media
role, never both media roles. Tarpit is not implemented. The internal coordinator
stays on do3, and both Macs host fossil subscribers, not coordinator deployments.
The public environment remains separate. Users open the coordinator URL to
manage their nodes; media paths and bytes remain node-local. Show these roles
without requiring users to understand container internals.

The first deployment starts a coordinator and its initial, pre-approved fossil node together. The first passkey registration creates the owner account for that coordinator instance. Later nodes do not select a user: the coordinator associates each pending request with its sole account and shows it only to that account's signed-in owner for explicit approval. Enrollment is rejected until the first account exists and while multiple accounts are unsupported. A future multi-account coordinator must use a short-lived owner-issued invitation to bind a new node to one account; it must not trust a user ID supplied by the joining node.

Storage scans run on their storage node at a persisted owner-configurable wall-clock time. The Devices card supports daily or weekly cadence, time, and an IANA time zone; it shows the next run and whether a scheduled scan is currently active. Store schedule state only in node-local `/data`. Route reads and updates directly or through the coordinator's owner-authenticated storage-only allowlist; the coordinator must not gain media access or own the subscriber schedule.

Device display names are subscriber-owned private configuration. Store them in the shared age-encrypted config keyed by stable node ID. The coordinator browser decrypts and re-encrypts this config locally; the coordinator service and PostgreSQL receive only age ciphertext and must never persist or log display names.

A device being online or connected through Syncthing or Tailscale does not mean it is trusted to access every user's data. Explain what a sharing action will expose and to whom. New invitees begin untrusted; the owner must explicitly choose what to share. Tailscale is an optional app-data/control transport and must never be presented as a media replication or media streaming feature.

For two owner-approved boneyard nodes, the target layout publishes a selected media collection under `/shared/out` for Syncthing replication. Keep the original `/media` source mount read-only and do not mirror it automatically. The current workflow still uses a separate Syncthing media share rooted at `/media`; moving it into `/shared/out` requires a planned migration. Syncthing transport encryption does not encrypt replicated files at rest. Use explicit folder authorization, matching ignore rules when only part of a collection should transfer, an empty destination for a new share, capacity checks, and file-hash verification. Node enrollment and Syncthing folder authorization remain separate owner actions.
The user-facing node names are **fossil** for metadata nodes and **boneyard** for storage nodes. **Tarpit** is the planned name for a future untrusted Syncthing destination that receives selected media from a boneyard and stores it encrypted at rest. The name evokes a dark, difficult-to-escape place, fitting data that the destination cannot view. This is distinct from the trusted two-boneyard replication described above. Tarpit is not implemented, and no implementation work has started; Syncthing transport encryption alone is not destination-at-rest encryption.

The internal coordinator stays on do3, separate from Mac fossil subscribers.
Keep coordinator HTTPS and subscriber/Syncthing paths distinct. The coordinator
shares its dedicated Tailscale namespace; Serve proxies HTTPS to coordinator
loopback. Configure Syncthing peers/folders only after owner approval and verify
their actual current routes, rather than replaying retired Mac coordinator
instructions. PostgreSQL stays on the coordinator's private Docker network,
never through Tailscale. Subscriber identity state remains node-local.
See [Tailscale integration](tailscale-integration-plan.md).

Show synchronization states in user terms: up to date, syncing, waiting for device, conflict needs review, or error. Do not label a node healthy just because its network connection is online.

## Feedback, errors, and accessibility

- Use concise confirmation after successful actions and state the actual result, not merely that a request was sent.
- Make errors actionable: name the failed service or step, explain the likely cause when known, and give the next safe action.
- Keep connection and synchronization status available without disrupting the page. Prefer server-sent updates over periodic full-page polling.
- Support keyboard navigation, visible focus, screen-reader labels, sufficient contrast, and text alternatives for identity logos.
- Do not rely on color alone to communicate identity or status.
- Avoid displaying secrets in errors, notifications, logs, browser titles, or copied diagnostic output.

- Preserve the selected workspace page in the URL so browser refresh and Back/Forward retain the expected view.

- Movie posters and titles open the matching Radarr movie in a new tab only after the browser can contact the configured Radarr URL. Failed, blocked, or timed-out checks leave plain content; refresh retries the check. Reachability does not imply authenticated access.

## Keeping this guide current

When the user recommends or approves a change to how Rawrarr should look or behave, update this guide in the same task. Keep decisions here even when implementation is still pending, and track completion in the [roadmap](../ROADMAP.md) until the UI/service supports them. Do not silently turn an implementation detail into a product decision.

- Movie quality-profile names come from shared metadata first. Resolve missing names from Radarr and persist them in encrypted movie metadata; refresh the cached name when the assigned profile ID changes.

## Metadata filename privacy

The main age-encrypted config contains `metadata_filename_key`, a dedicated 256-bit secret generated by the operating system CSPRNG and reserved exclusively for opaque metadata filename derivation. It is independent of age identities, credentials, and authentication keys. Preserve it through Radarr edits and removal; never include it in plaintext Radarr mirrors, UI responses, or logs. Nodes must share the same provisioned config before using it. Movie text and artwork filenames use separate domain-separated HMAC-SHA-256 inputs over the TMDb ID and retain `.json.aged` and `.aged` suffixes. Existing numeric ciphertext is copied byte-for-byte to the HMAC path, verified, and then its numeric filename is removed; readers accept legacy numeric paths until migration finishes. Never rotate this key after opaque files exist.

## Mac mini boneyard deployment direction

The Mac mini runs a boneyard worker in OrbStack, joining the MacBook roarlay instance and scanning `/Volumes/silo/Media/data` read-only. It needs its own device identity and operational state, but no PostgreSQL, user passkeys, or private age identity for encryption-only scanning. Preserve the shared instance and public age recipient. See the [deployment plan](mac-mini-storage-node-plan.md); persistent worker, enrollment, and catalog integration remain planned.

## Approve a node with an existing signed-in node

The new node generates and retains its random credential in local `/data`, then sends a short-lived pending request containing only its node ID, role, display name, browser origin, RP ID, and a one-way credential verifier. It does not send its age identity, media, shared data, or node-local app data. An already signed-in node lists pending requests and lets its user explicitly approve one. The requesting node polls with its local credential and registers only after approval. Requests expire after 15 minutes. Show pending, approved, expired, rejected, and coordinator-offline states clearly. Device approval is separate from local age identity setup and never implies that the coordinator can decrypt subscriber data.

## Responsive movie loading

Render movie rows and begin loading posters as soon as the movie API responds. Check browser access to Radarr independently and enable detail links afterward. Keep the previous list visible during an in-page refresh and explicitly label it as previous data if updating fails. Do not persist movie metadata in browser storage.

## Movie identifiers

The Movies table displays the movie title without appended identifiers. Its existing TMDb identifier appears in a dedicated TMDb ID column; missing IDs display an em dash.

Age private-ID provisioning for the Mac mini is a separate local setup step with hidden terminal input and a protected read-only secret mount. Do not request or transfer it through the web UI or device-linking process. Device approval and local vault readiness are separate statuses; neither implies the other.

Movie metadata decryption is memoized only in the unlocked subscriber vault process. Ciphertext content changes invalidate entries; cold reads use bounded concurrency. Shared encrypted files remain authoritative.

Movies reloads show the active subscriber connection’s last successful list immediately, labeled with its update time while a live request runs. This memory-only snapshot is discarded when the connection is replaced or cleared. Live results always take precedence over late snapshots.

## Shared Arr library presentation

Radarr and Sonarr use one `libraryColumns` schema and `renderArrLibrary` renderer in `rust-arr/src/ui.html`. Column order, cell formatting, artwork behavior, and table layout changes apply to both. Provider descriptors supply labels, IDs (TMDb/TVDB), detail paths, and availability (movie file/episode counts). Keep future app adapters in this registry instead of copying tables.

Sonarr uses the shared Arr HTTPS transport, validates service identity before saving, and stores its connection in the subscriber age-encrypted config. Locked read-modify-write operations preserve other app connections and the metadata filename key. Metadata nodes scrape uncached Sonarr series metadata and posters sequentially after connection, skip age-encrypted shared cache hits, and repeat after six hours. Metadata search and episode mutations remain planned.

### Prowlarr discovery adapter

Treat Prowlarr as a replaceable discovery adapter, not Rawrarr's library or fulfillment system of record. Reuse the shared Arr transport and coordinator service-settings flow. The selected metadata node owns the URL and API key in the age-encrypted config; remote instances require HTTPS, while HTTP is accepted only for node loopback. The coordinator browser sends settings directly to that node, and roarlay receives no Prowlarr credentials. Show normalized indexer status and owner-initiated, bounded search results with stable IDs and indexer provenance. Do not expose download URLs or add automatic downloads/fulfillment as part of discovery.

Offer an owner-run Compose overlay instead of giving the node application Docker-socket access. The overlay shares the fossil's Tailscale namespace, stores Prowlarr configuration in a node-local Docker volume, and publishes the setup UI only on host loopback. Applying it can recreate the Tailscale sidecar and interrupt the node; explain that risk and require review, a maintenance window, and preservation of existing volumes. For adoption, keep the external instance untouched: export a native Prowlarr backup, restore it only into a fresh managed instance of a compatible version, compare configuration there, test the target connection before changing Rawrarr's saved connection, and retain the source URL/API key for rollback. Backup archives may contain credentials and must stay outside roarlay and synchronized data.

The shared config uses the versioned [schema v1](shared-config-v1.schema.json). Store Radarr and Sonarr connections as typed entries in the `services` array; readers accept legacy flat Radarr/Sonarr fields and rewrite them as schema v1 on unlock. Keep service credentials inside age ciphertext and redact them in browser-rendered config.


Artwork load times measure each image request from browser `requestStart` through `responseEnd`, excluding lazy-loading wait, browser queueing, and decoding. Missing or restricted timing is shown as unavailable, never as elapsed time since rendering.

## Metadata export

Let an owner export the shared metadata tree as useful plaintext files, with artwork as the primary expected use. Decrypt age files in the user's browser and build the archive there; the server may authorize and deliver ciphertext, but must not receive the age secret or decrypted payload for this feature. Require an age identity in browser memory, keep passkey authentication distinct from decryption, and clearly report corrupt or unsupported files. Keep this export separate from a full Rawrchive backup and exclude configuration, logs, databases, node-local state, and media. The detailed format, security boundaries, and implementation sequence are in the [metadata export plan](metadata-export-plan.md). Settings currently exports all indexed movie text records and matching available artwork in an in-memory ZIP; other metadata classes, streaming scale, cancellation, and full failure reporting remain planned.
