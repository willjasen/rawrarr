# Rawrarr product and interface design

This is the living product-design guide for how Rawrarr should behave and feel to its users. It captures user-facing decisions and expectations, rather than implementation details. For technical boundaries and mechanisms, see [data domains](data-domains.md), [device architecture](device-architecture.md), and [technical status](technical-status.md). For how to use the current prototype, see the [user guide](user-guide.md).

## Design goals

- **Private by default.** Make it clear which information is public, encrypted, shared, or local. Do not reveal private identities, credentials, media paths, or full identifiers in ordinary UI.
- **Friendly and calm.** Use the Rawrarr dinosaur identity and green accents with a clean, modern interface. Keep setup focused and avoid overwhelming first-time users with advanced configuration.
- **Show useful status.** Tell the user whether an integration or device is connected, when information last changed, and what action can fix a problem. Prefer live updates over asking the user to reload.
- **Keep control with the user.** Make sharing, scanning, backup, and connection actions explicit. Explain the effect before an action that changes or sends data.
- **Remain understandable when nodes are offline.** Show which data is local or shared, which device can perform an action, and whether it is waiting for a peer to reconnect.
- **Support sharing beyond media.** Treat device-to-device file sharing as a core direction for Rawrarr, following the broader Synkpieces idea. Users should eventually be able to share ordinary files and folders; media automation is the first focused use case, not a required data type.

## Visual identity

Rawrarr should feel approachable and distinctive without making the interface look like a toy. Use a restrained dark or light surface system, readable typography, clear section hierarchy, and green as the primary brand accent. Reserve bright colors for status, identity illustrations, and meaningful actions.

Serve the brand logo as a lossless JPEG XL image where the browser supports it, with a PNG fallback. Future logo animation may be used sparingly for selected moments; keep a static fallback and respect reduced-motion preferences.

Always render the product name as lowercase **rawrarr** in user-facing interface text, errors, and logs.

A generated identity illustration and the active workspace color scheme should be derived deterministically from the public age recipient. The same recipient should always produce the same logo and palette across nodes. Keep the normal Rawrarr brand palette before an identity is created or unlocked; switch to the recipient-derived palette after creation or successful import. Use a recognizable multicolor geometric identity mark and a coordinated accent/surface tint in the app. Never place the full recipient or private identity inside the illustration or its accessible label. The logo and palette identify an age identity visually, while the displayed short form lets a person distinguish nearby identities. Preserve contrast and semantic status colors so color alone never communicates status.

## Account and first-run flow

1. Present a minimal sign-in page centered on passkey registration or sign-in.
2. After a first passkey is registered, take the user to the age setup screen before the main app.
3. Let the user create a new age identity or paste an existing one into a masked field. Explain that the identity protects shared configuration and metadata. Keep copy confirmation distinct from unlocking the encrypted vault.
4. Show only shortened identity forms. Never render the complete private identity or complete public recipient in page text, labels, tooltips, or logs.
5. After the user selects **Create ID**, open a pop-up that guides them through copying only the private key line. Show the acknowledgement checkbox only after the clipboard copy succeeds. Label it **I copied it and saved the private ID somewhere safe.** Once checked, ask for three characters at random positions after the fixed key prefix; when the correct characters are entered, unlock the vault and open the main workspace automatically. Show only the first 12 and last 5 characters of the generated identity with an ellipsis. Keep the challenge and typed characters in page memory only, clear them when the identity changes or the page closes, and do not reveal the full identity to prove that generation succeeded. This verifies the user can read the copied key but does not prove where they stored it.
6. Show the recipient-derived colored logo and only the first 12 and last 8 characters of the public recipient with an ellipsis. Never render the full recipient in page text, labels, tooltips, or logs. Keep the full private identity available only for the copy action and the deliberate unlock operation.
7. Let users paste an existing identity into the masked field; do not provide a file-selection action. Use the same copy pop-up, acknowledgement, and character challenge for pasted identities, then let the user select **Unlock vault** to unlock the encrypted settings. Keep the unlock button out of the generated-ID flow so the correct character entry continues automatically. Do not call the final action “Import and unlock.”
8. After the vault is unlocked, take the user to the main workspace. On a later session, require passkey sign-in and age-vault unlock again unless the user explicitly configured a protected local secret.

The shortened display is an explicit privacy decision: partial forms are for recognition; they are not a substitute for the complete identity and must not be expandable inline.

## Main workspace

The main workspace should make the user's shared files, media, and service health easy to scan at a glance. The first version can stay focused on the media library, but the workspace should not imply that all shared data must be a movie, episode, album, or book. Keep the primary areas recognizable:

- **Library:** files and folders shared through Rawrarr, with media views for movies and other recognized media. Show clear file, metadata, and availability states without requiring every item to have media metadata.
- **Activity:** recent scans, metadata updates, service events, and actionable errors.
- **Devices:** nodes and their roles, connectivity, shared-folder status, and the data each device can access.
- **Settings:** connected Arr services, metadata sources, and app preferences.

Use plain language in empty states. When there is no library content, direct the user to configure a media folder and scan it. When an integration is disconnected, show its last known state and a specific next step.

## Integrations and library behavior

### Radarr

Treat Radarr as the first supported external library integration. Let the user enter its URL and API key in the UI, validate the connection, and apply saved settings without requiring an app restart. Show connection status and movie data in the workspace. Mask or redact the API key everywhere it is displayed after entry.

### Local scans and metadata

Let the user choose a mounted media location and start a scan deliberately. Explain that a scan reads media and creates Rawrarr metadata; it does not move or rename media files. Show scan progress and per-item matching status. When a match is uncertain, present choices rather than silently attaching incorrect metadata.

Metadata and artwork that Rawrarr shares should be identified as encrypted. Explain when the app cannot create an encrypted sidecar because the shared public recipient is unavailable. Keep media files themselves distinct from metadata and app data in every screen.

### Future discovery and fulfillment

Rawrarr is intended to grow beyond Radarr into discovery, library management, and fulfillment. Present these as capabilities only when they exist. Keep provider-neutral concepts understandable to users and make the selected search or transfer provider visible when that affects a request. Do not imply that Rawrarr currently downloads or imports content through a concrete client when it does not.

## Devices and sharing

Use the words **storage node** for a Rawrarr device with local media and **shard node** for a media-less Rawrarr web node. The operator's SwarmSail service node is a separate operational role. Make each device's role visible, but avoid requiring users to understand container internals to manage their setup.

A device being online or connected through Syncthing or Tailscale does not mean it is trusted to access every user's data. Explain what a sharing action will expose and to whom. New invitees begin untrusted; the owner must explicitly choose what to share. Tailscale is an optional app-data/control transport and must never be presented as a media replication or media streaming feature.

Show synchronization states in user terms: up to date, syncing, waiting for device, conflict needs review, or error. Do not label a node healthy just because its network connection is online.

## Feedback, errors, and accessibility

- Use concise confirmation after successful actions and state the actual result, not merely that a request was sent.
- Make errors actionable: name the failed service or step, explain the likely cause when known, and give the next safe action.
- Keep connection and synchronization status available without disrupting the page. Prefer server-sent updates over periodic full-page polling.
- Support keyboard navigation, visible focus, screen-reader labels, sufficient contrast, and text alternatives for identity logos.
- Do not rely on color alone to communicate identity or status.
- Avoid displaying secrets in errors, notifications, logs, browser titles, or copied diagnostic output.

## Keeping this guide current

When the user recommends or approves a change to how Rawrarr should look or behave, update this guide in the same task. Keep decisions here even when implementation is still pending, and track completion in the [roadmap](../ROADMAP.md) until the UI/service supports them. Do not silently turn an implementation detail into a product decision.
