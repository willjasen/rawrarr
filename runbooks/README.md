- [GitHub Actions test workflow](2026-10-20-github-actions-tests.md) — Run lightweight standard-library script tests automatically on pull requests and main-branch pushes.

- [Implement the metadata-node Redis artwork cache](2026-10-18-implement-metadata-redis-cache.md) — Cache encrypted movie and TV artwork in optional node-local Redis with shared-file fallback.
- [Fix large TV artwork pipe deadlocks](2026-10-18-fix-large-tv-artwork-pipe.md) — Drain age subprocess output concurrently with large artwork input so poster requests do not hang.
- [Update Mac mini Rawrarr containers](2026-10-18-update-mac-mini-containers.md) — Refresh the Mac mini storage image and relay proxies, verifying all preserved mounts and routes.
- [Fix Overview metadata node selection](2026-10-18-fix-overview-metadata-node-selection.md) — Use an available metadata node when an apex node is selected and clarify the Overview empty state.
- [Coordinator links to reachable Arr entries](2026-10-18-coordinator-arr-detail-links.md) — Show Radarr and Sonarr year-cell links only after browser-side service reachability succeeds.
- [Remove device name helper text](2026-10-18-remove-device-name-helper.md) — Remove the instructional copy beneath coordinator device cards while preserving editing and navigation.

- [Show shared-folder health in the coordinator Devices view](2026-10-18-device-shared-folder-health-coordinator.md) — Route per-node Syncthing folder and peer status into coordinator device cards; verify the authenticated live view.

- [Show shared-folder health in Devices](2026-10-18-device-shared-folder-health.md) — Report Syncthing folder and peer replication state in the Devices panel.

- [Deploy the TV menu label to roarlay](2026-10-18-deploy-tv-menu-to-roarlay.md) — Correct the coordinator Sonarr section label and page heading, then deploy and verify roarlay.
- [Rename the Series menu to TV](2026-10-18-rename-series-menu-to-tv.md) — Use “TV” for the Sonarr section in navigation and its page heading.
- [Add a sidebar to the coordinator workspace](2026-10-18-coordinator-workspace-sidebar.md) — Add responsive Overview, Movies, Series, Configuration, and Settings navigation while preserving the current coordinator layout.

- [Format device shared-folder status](2026-10-18-format-device-shared-folder-status.md) — Remove the Syncthing folder ID from device cards and format state, completion, and peer connectivity clearly.

- [Show peer counts and device topology](2026-10-18-device-topology-and-peer-count.md) — Include peer counts on matched device statuses and show Syncthing links in a topology section below the device list.
- [Improve the device topology diagram](2026-10-18-improve-device-topology-diagram.md) — Show enrolled devices in a network diagram with one line per device pair and live peer-state coloring.

# Rawrarr runbooks

- [Plan one-line node deployment](2026-10-20-plan-one-line-node-deployment.md) — Plan a generated embedded-Compose installer, Docker setup, safe retries, and explicit owner approval; no runtime changes.

- [Name fossil and boneyard nodes](2026-10-20-fossil-and-boneyard-terminology.md) — Use fossil for metadata nodes and boneyard for storage nodes while retaining internal role identifiers.

- [Headscale container client staging](2026-10-18-headscale-container-client-staging.md) — Add isolated Docker Tailscale clients on both Macs to Headscale through the Caddy coordinator route, preserving current app sidecars during the transition.

- [Subscriber node onboarding on roarlay](2026-10-18-subscriber-node-onboarding.md) — Guide passkey owners through local age/root backup and offer subscriber-only Compose deployment instructions.

- [Add three music albums to the MacBook replica](2026-10-18-add-three-music-albums.md) — Expand the existing Syncthing filter to three selected album directories and verify their transfer.

- [Media `.ev` folder layout](2026-10-18-media-ev-folder-layout.md) — Rename the MacBook's media replica root and reserve a movies-only folder for future selective sync.

- [Public Rawrarr landing page](2026-10-18-public-landing-page.md) — Serve a security-qualified product introduction before passkey sign-in while keeping the authenticated workspace at the coordinator root.

- [Integrate subscriber tailnets with Swarmsail](2026-10-18-swarmsail-subscriber-tailnet-integration.md) — Record the eligibility gate, one-way privacy rule, current Swarmsail change, and remaining hosted integration.

- [Storage media replication](2026-10-18-storage-media-replication.md) — Add explicit first-to-second boneyard media-folder sharing through Syncthing.

- [Fix TV artwork serving with Redis enabled](2026-10-18-fix-tv-artwork-redis.md) — Diagnose and restore the MacBook metadata node's encrypted TV poster responses while keeping Redis enabled.

- [Document Arr summary cards](2026-10-18-document-arr-summary-cards.md) — Record shared card rendering, service-specific metadata sizes, and completed Sonarr backfill visibility in project docs.

- [Share Arr summary card rendering](2026-10-18-shared-arr-summary-cards.md) — Use one configurable four-card renderer for Radarr, Sonarr, and future Arr service summaries.

- [Move Arr detail link beside title](2026-10-18-move-arr-link-beside-title.md) — Place the reachable-service external link beside the title and year in the first Movies/TV column.

- [Hide TV backfill status when posters are saved](2026-10-18-hide-tv-complete-backfill-status.md) — Treat a fully saved Sonarr artwork cache as complete even while the worker checks for new series.

- [Fix TV metadata size](2026-10-18-fix-tv-metadata-size.md) — Report the TV metadata subtree separately from Movies and all metadata.

- [Rescan metadata from Movies and TV](2026-10-18-metadata-rescan-buttons.md) — Let Movies and TV request an on-demand encrypted metadata refresh from the selected metadata node.

- [Isolated to shared cluster design](2026-10-18-isolated-to-shared-cluster-design.md) — Record owner-controlled membership, one-way sharing, age multi-recipient encryption, MLS evaluation, and revocation cleanup status.

- [Complete Radarr and Sonarr text metadata](2026-10-18-complete-arr-text-metadata.md) — Store full Arr text metadata as age-encrypted HMAC-named JSON, separate from storage-node observations.

- [Clarify and deploy the storage media scan action](2026-10-18-clarify-storage-scan-action.md) — Label the storage-node media scan accurately and deploy the coordinator UI to roarlay.

- [Show storage scan progress](2026-10-18-storage-scan-progress-indicator.md) — Show an accessible indeterminate progress bar while the storage-node scan is running.

- [Report storage scan item progress](2026-10-18-storage-scan-item-progress.md) — Show completed and total media files as “1 of 3” while a storage scan runs.

- [Stop a storage scan](2026-10-18-stop-storage-scan.md) — Add an owner-controlled Stop scan action that safely cancels after the current media file.

- [Browser-triggered and daily storage scans](2026-10-18-browser-triggered-storage-scans.md) — Let the owner request storage scans from Devices and check missing encrypted metadata daily.

- [Overview shared metadata and Arr statistics](2026-10-18-overview-shared-metadata-arr-stats.md) — Show synchronized-folder health, shared metadata size, and Radarr and Sonarr library totals on Overview.

- [Small-file sidecar scan test](2026-10-18-small-file-sidecar-scan-test.md) — Record the Mini access checks and why a bounded 1–2 GB sample scan could not be run safely yet.

- [Show Sonarr series artwork on TV](2026-10-18-show-sonarr-series-artwork.md) — Serve cached age-encrypted TV posters through the direct metadata API and display lazy thumbnails in the TV series table.

- [Hide completed Sonarr scan messages](2026-10-18-hide-sonarr-complete-scrape-messages.md) — Hide the completed artwork-cache status and successful series-load message after the TV scan finishes.

- [Rust encrypted media sidecars](2026-10-18-rust-encrypted-media-sidecars.md) — Add file hashes and ffprobe details to age-encrypted storage-node records in the shared metadata tree.

- [Automatic scanning on storage nodes](2026-10-18-storage-automatic-scanning.md) — Run bounded age-encrypted library scans after storage-node registration and repeat on a configurable interval.

- [Profile name two-row layout](2026-10-18-profile-name-two-row-layout.md) — Join the four generated words without separators and show color/trait above the base-name words.

- [Automatically backfill Sonarr metadata and artwork](2026-10-18-automatic-sonarr-metadata-backfill.md) — Scrape uncached Sonarr series metadata and posters on first connection and every six hours using the metadata node.

- [Fix the direct Sonarr library 404](2026-10-18-fix-direct-sonarr-library-404.md) — Expose the protected Sonarr read endpoints through the direct metadata API used by the TV page.
- [Color and trait account name](2026-10-18-color-and-trait-account-name.md) — Prefix generated profile names with the avatar's main color and wing trait while preserving the username length limit.

- [Fix the direct Sonarr library 404](2026-10-18-fix-direct-sonarr-library-404.md) — Expose the protected Sonarr read endpoints through the direct metadata API used by the TV page.

- [Browse the Sonarr library on TV](2026-10-18-browse-sonarr-library.md) — Load the configured Sonarr series list directly from metadata and show TV library statistics and refresh controls.

- [Add suggested account words](2026-10-18-add-suggested-account-words.md) — Expand generated names with the previously suggested eras, settings, and creatures.
- [Expand generated account vocabulary](2026-10-18-expand-generated-account-vocabulary.md) — Add the supplied prehistoric word pools while preserving username limits.

- [Display the generated account name](2026-10-18-display-generated-account-name.md) — Show the generated dinosaur-themed label in the workspace profile for existing and new accounts.

- [Fix profile avatar display](2026-10-18-fix-profile-avatar-display.md) — Keep profile avatars visible when the metadata node cannot provide the public age ID, and clip the Settings image to its circle.

- [Generate a dinosaur-themed account name](2026-10-18-generate-dinosaur-account-name.md) — Remove user-entered naming from first passkey setup and generate an account name with prehistoric words.

- [Show the signed-in profile in the sidebar](2026-10-18-show-profile-in-sidebar.md) — Replace the coordinator sidebar tagline with the account's generated picture and display name.

- [Unlock Movies with the age ID already loaded in Settings](2026-10-18-unlock-movies-with-loaded-age-id.md) — Automatically unlock the Movies library when navigating there with an age ID already in browser memory.

- [Add an About page](2026-10-18-about-page.md) — Introduce rawrarr, its current media and encrypted metadata capabilities, and its future sharing direction.

- [Prevent a flash of the configured Radarr form](2026-10-18-prevent-configured-radarr-form-flash.md) — Start Movies and TV connection cards hidden and reveal them only when settings report that setup is needed.

- [Hide configured Radarr and Sonarr connection forms](2026-10-18-hide-configured-arr-forms.md) — Keep media-tab connection cards visible only until the corresponding service has saved settings; manage configured services in Settings.

- [Prompt for the age ID from Settings metadata export](2026-10-18-settings-export-age-id-prompt.md) — Let Settings export prompt for the age identity when the tab has no in-memory key.

- [Remember the age private ID in this browser](2026-10-18-remember-age-id-in-browser.md) — Offer opt-in browser persistence for the age ID while warning that tab-only memory is the higher-security option.

- [Format Arr connection panels on Movies and TV](2026-10-18-format-arr-connection-panels.md) — Provide consistent Radarr and Sonarr URL/API-key forms on their media tabs, routed directly to metadata.

- [Put complete movie metadata export in Settings](2026-10-18-settings-metadata-export.md) — Move browser-side export into Settings and include every indexed movie record and matching artwork.

- [Use the TV route and open Sonarr settings](2026-10-18-use-tv-route-and-open-sonarr-settings.md) — Make `#tv` canonical and link the TV page to the Sonarr URL/API-key form.

- [Version shared config and group services](2026-10-18-version-shared-config-schema.md) — Define schema v1, store Radarr and Sonarr in `services`, and migrate legacy config while keeping browser writes compatible.

- [Show redacted cluster configuration](2026-10-18-show-redacted-cluster-config.md) — Decrypt shared age config in the browser through the ciphertext relay and redact secrets in the Configuration view.

- [Prefer directly connected metadata nodes](2026-10-18-prefer-direct-metadata-node.md) — Choose an online metadata node with a direct route before a relay-only node when roarlay picks a default.

- [Show test timing and clean old test containers](2026-10-18-test-run-timing-cleanup.md) — Add start, stop, and elapsed time to test summaries and periodically remove stopped test containers older than one hour.

- [Edit device names in the coordinator](2026-10-18-age-encrypted-device-names.md) — Rename approved nodes in the coordinator UI while keeping names in browser-decrypted age config and out of coordinator service storage.

- [Restore the coordinator user profile](2026-10-18-restore-user-profile-picture.md) — Show the passkey account's saved display name and a deterministic, locally generated profile picture in Settings.

- [Generate a dinosaur profile picture from the public age ID](2026-10-18-age-id-dinosaur-profile-picture.md) — Render a distinct, repeatable dinosaur avatar in the browser from the public age recipient, without storing image data in PostgreSQL.

- [Vary dinosaur profile spikes](2026-10-18-vary-dinosaur-profile-spikes.md) — Allow zero to three identity-derived back spikes, with larger single-spike and distinctly sized two-spike variants.

- [Add variable tails and optional wings](2026-10-18-add-dinosaur-tail-and-wings.md) — Give generated profile dinosaurs identity-derived tail lengths and optional folded wings.
- [Add a rare golden dinosaur profile trait](2026-10-18-add-rare-golden-dinosaur.md) — Give a small, deterministic share of public-age-ID-generated profile dinosaurs a gold palette and sparkle accents.
- [Remove back spikes from winged dinosaurs](2026-10-18-no-spikes-on-winged-dinosaurs.md) — Omit back spikes whenever the generated dinosaur has wings.
- [Add dinosaur background scenes](2026-10-18-add-dinosaur-background-scenes.md) — Render volcanic, forest, or hilly scenery behind generated profile dinosaurs.
- [Improve dinosaur background scenes](2026-10-18-improve-dinosaur-background-scenes.md) — Improve forest tree visibility and add desert, snowy, and tropical identity-derived avatar scenes.
- [Serve dinosaur avatars as JPEG XL](2026-10-18-serve-dinosaur-avatars-as-jxl.md) — Generate profile art on roarlay from the public-age-ID digest and return raster JPEG XL with a PNG browser fallback.

- [Remove the profile avatar status sentence](2026-10-18-remove-profile-avatar-status-copy.md) — Clear the success copy after the generated profile avatar loads, while retaining loading and error feedback.

- [Remove the profile name helper text](2026-10-18-remove-profile-name-helper.md) — Remove the redundant passkey setup helper line beneath the coordinator profile name.

- [Show direct, relay, or offline node status](2026-10-18-direct-relay-offline-node-status.md) — Allow direct-only nodes to open and show whether each online node uses a direct Tailscale route or coordinator relay.

- [Keep the selected coordinator node out of the browser URL](2026-10-18-hide-selected-node-query.md) — Preserve node selection across refreshes in per-tab storage while keeping the address bar free of the `node` query parameter.

- [Style the coordinator route status](2026-10-18-style-route-status-indicator.md) — Make checking, direct, relay, and fallback connection states easier to scan while preserving screen-reader announcements.

- [Fix coordinator movie library clearing during node polling](2026-10-18-fix-coordinator-movies-refresh.md) — Keep the decrypted Movies view stable while refreshing coordinator node status.

- [Hide age ID entry controls when loaded](2026-10-18-hide-age-id-controls-when-loaded.md) — Hide the age private ID entry and unlock controls while a key is saved in this browser profile, retaining an explicit clear action.

- [Implement the Tailscale direct node route](2026-10-18-implement-tailscale-direct-node-route.md) — Route eligible browser metadata requests directly to an allowlisted metadata API over Tailscale Serve, show automatic route selection, and retain coordinator fallback.

- [Set the coordinator browser tab name](2026-10-18-coordinator-browser-tab-name.md) — Use the lowercase product name as the coordinator workspace document title.

- [Share the age private ID across browser tabs](2026-10-18-share-age-key-across-browser-tabs.md) — Keep the age private ID in browser-profile storage so new tabs can unlock locally, and clear it on request or sign-out.

- [Plan optional direct node connections](2026-10-18-plan-optional-direct-node-connections.md) — Plan browser-to-node connections when a node is reachable directly while retaining the coordinator route for fallback.

- [Isolate metadata and storage containers from host access](2026-10-18-isolate-node-containers-from-host.md) — Remove host-published node ports and bind node listeners to loopback inside their Tailscale namespaces, preserving coordinator and tailnet relay paths.

- [Fix encrypted library response authentication](2026-10-18-fix-encrypted-library-authentication.md) — Align browser and metadata-node ECDH/HKDF key derivation so the browser can authenticate and decrypt the relayed Radarr library response.

- [Fix the encrypted library relay 502](2026-10-18-fix-library-relay-502.md) — Point the coordinator's trusted metadata relay at the active recovery service and verify the coordinator route.

- [Reconnect the MacBook metadata namespace](2026-10-18-reconnect-macbook-metadata-namespace.md) — Reattach a healthy but stale metadata container to the active Tailscale namespace and restore direct and relay health.

- [Show the full Movies page through the coordinator](2026-10-18-full-coordinator-movies-page.md) — Replace the three-record card proof of concept with the usual full Radarr movie table while keeping its inventory encrypted through the coordinator.

- [Use a ciphertext-only coordinator relay](2026-10-18-ciphertext-coordinator-relay.md) — Replace the coordinator's plaintext-capable node page proxy with a bounded ciphertext relay and browser-side movie rendering.

- [Blind coordinator content boundary](2026-10-18-blind-coordinator-content.md) — Record the ciphertext-only coordinator requirement, browser decryption target, and current plaintext-capable node-page relay gap.

- [Make coordinator node status dots visible](2026-10-18-make-node-status-dots-visible.md) — Enlarge online/offline indicators and increase their contrast after the user reported not seeing the dots.
- [Restore the Mac mini node relay route](2026-10-18-fix-mini-node-relay.md) — Restore its containerized Tailscale path and configure a coordinator-reachable, tailnet-only storage page route.

- [Enable the Mac mini direct Tailscale route](2026-10-18-enable-mini-direct-tailscale-route.md) — Add a cross-origin, short-lived authenticated direct API path over the Mini's Tailscale Serve while preserving coordinator relay fallback.

- [Hide coordinator passkey migration on the tailnet](2026-10-18-hide-tailnet-passkey-migration.md) — Show the migration action only on the local metadata origin.

- [Show node online and offline status in the coordinator](2026-10-18-coordinator-node-status-indicators.md) — Add explicit, accessible green and red status indicators to connected-node buttons.

- [Serialize Rawrarr container deployments](2026-10-18-serialize-container-deployments.md) — Use a hidden marker outside synchronized shared data to ensure only one task rebuilds or deploys containers at a time.

- [Clarify Syncthing approval-gated onboarding](2026-10-18-syncthing-approval-gate-documentation.md) — Align architecture, data-domain, technical-status, and Mini storage-node documentation so a new production node receives no Syncthing peer or folder configuration before explicit owner approval.

- [Use Syncthing public relays and gate node sharing on owner approval](2026-10-18-syncthing-relay-approval-gate.md) — Use public relay fallback for the two-Mac test and document that production nodes must receive explicit owner approval before Syncthing peers or folders are configured.

- [Rename the workspace menu label](2026-10-18-rename-workspace-menu.md) — Change the left navigation section label to “rawrchive”.

- [Run the coordinator in the Tailscale namespace](2026-10-18-merge-coordinator-proxy.md) — Remove the Caddy coordinator proxy and have Tailscale Serve target the coordinator directly on loopback.

- [Document Tailscale coordinator and Syncthing routes](2026-10-18-document-tailscale-proxy-routes.md) — Clarify why coordinator HTTPS uses a loopback bridge while Syncthing peer TCP forwards directly, and keep the Syncthing GUI local.

- [Remove the Syncthing TCP proxy](2026-10-18-remove-syncthing-proxy.md) — Remove the unused bridge after confirming Tailscale Serve forwards peer TCP directly to Syncthing.

- [Show connected node pages from the coordinator](2026-10-18-coordinator-node-workspace.md) — Sign in at the coordinator, list owner-authorized nodes, and display a selected node through a restricted relay.

- [Remove Movies Path column](2026-10-18-remove-movies-path-column.md) — Remove filesystem paths from the movie table.

- [Review current Rawrarr documentation](2026-10-18-documentation-review.md) — Align local setup, identity handling, artwork, health semantics, and testing instructions with the current source.

Task records and operational procedures for this Rawrarr repository. Each entry links to the specific task; older records are retained as history.

- [Disable UI pages on metadata and storage nodes](2026-10-18-disable-subscriber-node-ui.md) — Keep page implementation code while returning 404 for UI routes on subscriber roles.

- [Restore the coordinator sidebar logo](2026-10-18-restore-coordinator-sidebar-logo.md) — Show the Rawrarr logo image in the coordinator's left navigation menu.

- [Summarize failed container tests](2026-10-18-rawrarr-test-failure-summary.md) — End retained container logs with pass/fail status and the names of failed Node.js or Rust tests.

- [Test age setup in Chromium](2026-10-18-age-challenge-playwright.md) — Add real-browser coverage for the age identity backup and unlock flow inside the isolated Rawrarr Docker test environment.

- [Speed up local development builds](2026-10-18-faster-local-development.md) — Persist Cargo registry and compiler artifacts through BuildKit caches, and keep the persistent local runtime user-updated.

- [Fix the age challenge start position](2026-10-18-fixed-age-challenge-position.md) — Always start the sequential private-key character check at the 5th character after the fixed age prefix.

- [Rebuild rawrarr-local from the current checkout](2026-10-18-rebuild-rawrarr-local.md) — Rebuild and recreate only the local app service with its current configuration, then verify health and the served setup page.

- [Make the age identity challenge input visible](2026-10-18-rawrarr-visible-identity-challenge-input.md) — Style the three-character entry field and focus it when the copy acknowledgement reveals the challenge.

- [Use a sequential age identity challenge](2026-10-18-rawrarr-sequential-age-challenge.md) — Verify copied identity text with three consecutive characters starting randomly from the 4th–7th character after its fixed prefix.

- [Apply Radarr settings without restarting rawrarr](2026-10-17-apply-radarr-settings-without-restart.md) — Make it clear that Radarr credentials entered in the dashboard apply to the running server immediately. Initial implementation already replaces…
- [Push dashboard updates with server-sent events](2026-10-17-dashboard-server-sent-events.md) — Replace browser-side 30-second Radarr status polling with a server-sent events stream that pushes dashboard data. Initial state: the dashboard…
- [Download client abstraction](2026-10-17-download-client-abstraction.md) — Create a provider-neutral interface for download clients so rawrarr can later connect to BitTorrent, Usenet, and other transfer methods. The app…
- [Fix Rawrarr passkey option decoding](2026-10-17-fix-rawrarr-passkey-options.md) — Fix the sign-in error Cannot read properties of undefined (reading 'replace') reported on the passkey page. The existing [passkey onboarding…
- [Local media folder scan](2026-10-17-local-library-scan.md) — Add a usable, read-only local media inventory to the rawrarr dashboard, with a Docker deployment option. The dashboard already served on…
- [Make Radarr setup easy to find](2026-10-17-make-radarr-setup-obvious.md) — Make the existing Radarr connection form discoverable from the rawrarr dashboard. The form and API already exist, but the dashboard places the…
- [Movie metadata matching](2026-10-17-movie-metadata-matching.md) — Extend the local read-only media scan toward a Radarr-style movie library by adding catalog search and user-reviewed matches. The scanner…
- [Radarr dashboard UI](2026-10-17-radarr-dashboard-ui.md) — Replace the plain-text root experience with an actual browser UI for the Radarr-focused Rust app. The local server returned ok only from /health;…
- [Radarr first integration](2026-10-17-radarr-first-integration.md) — Add a read-only first Radarr integration to the Rust service while keeping future Arr integrations possible. The committed skeleton exposes only…
- [Rawrarr 24-hour trial direction](2026-10-17-rawrarr-24-hour-trial.md) — Record the user's trial requirement: the person who sets up Rawrarr may use it for 24 hours, after which the instance stays locked until…
- [Persist Radarr settings in age-encrypted config](2026-10-17-rawrarr-age-encrypted-config.md) — Persist the Radarr URL and API key from the Rawrarr UI in an age-encrypted config file, apply changes immediately, and load saved settings after…
- [Prompt for age identity during Rawrarr setup](2026-10-17-rawrarr-age-identity-onboarding.md) — Make first-run age identity setup visible immediately, including while the Movies view is selected. The identity form existed inside the Overview…
- [Rawrarr blind public relay direction](2026-10-17-rawrarr-blind-relay-direction.md) — Record the user's future requirement for a public Rawrarr domain that reaches a subscriber's self-hosted node without giving the relay access to…
- [Add a Rawrarr configuration integration test](2026-10-17-rawrarr-config-api-test-script.md) — Add a repeatable test script for the configuration and age-encryption API. Existing script tests are indexed in [the test…
- [Add decrypted configuration and age verification page](2026-10-17-rawrarr-configuration-page.md) — Show the currently readable configuration intended for Rawrarr peers and report whether its stored form is age-encrypted. Initially, Radarr…
- [Rawrarr device model](2026-10-17-rawrarr-device-model.md) — Introduce a first-class concept for computers that hold media and run scanners, separately from the rawrarr web process. Today the web process…
- [Store Rawrarr metadata artwork with age encryption](2026-10-17-rawrarr-encrypted-artwork.md) — Store movie poster art returned by metadata lookup as age-encrypted files using the .aged extension. Existing configuration encryption uses the…
- [Rawrarr passkey and PostgreSQL onboarding](2026-10-17-rawrarr-passkey-postgres-onboarding.md) — Add a minimal passkey-first sign-in flow, followed by age identity creation/import, then the existing dashboard. Persist passkey credentials and…
- [Rawrarr private self-hosted product direction](2026-10-17-rawrarr-private-self-hosted-direction.md) — Record the user's direction for how Rawrarr should be distributed and how a private owner may share with another person. Existing product…
- [Show the Radarr movie library in Rawrarr](2026-10-17-rawrarr-radarr-movies.md) — Expose Radarr's existing movie collection and useful file data in Rawrarr. The configured instance at 127.0.0.1:8080 reports Radarr 6.4.4 as…
- [Configure Radarr from the rawrarr UI](2026-10-17-rawrarr-radarr-ui-config.md) — Let the user configure Radarr from the local rawrarr dashboard while preserving the optional environment-based setup. Initial state: Radarr URL…
- [Store Rawrarr config as shared age ciphertext](2026-10-17-rawrarr-shared-age-config.md) — Put Rawrarr's durable settings in the host folder used for its Syncthing share and ensure the config is age encrypted. The code already encrypted…
- [Rawrarr shared instance identity](2026-10-17-rawrarr-shared-instance-id.md) — Record the user's requirement for one global, shared ID that identifies a particular Rawrarr folder across all nodes syncing it, distinct from…
- [Rawrarr storage-less web node](2026-10-17-rawrarr-storage-less-web-node.md) — Record the user's clarification that a web node without media mounts should still serve the Syncthing shared folder containing Rawrarr application…
- [Rawrarr Syncthing peer sync](2026-10-17-rawrarr-syncthing-peer-sync.md) — Add a Rawrarr integration that configures a web node and a storage node to share a folder using Syncthing, then verify a real folder and file…
- [Isolate the two-container Rawrarr test group](2026-10-18-rawrarr-testing-docker-group.md) — Put the two-container OrbStack integration test under its own `rawrarr-testing` Docker project identity while preserving the persistent `rawrarr-local` environment.
- [Run Rawrarr tests in Docker](2026-10-18-rawrarr-docker-test-runner.md) — Build and run self-contained Node and Rust checks in a unique container in the `rawrarr-testing` group, retaining logs after completion.
- [Document Rawrarr's unified product direction](2026-10-17-rawrarr-unified-product-direction.md) — Record the user's direction that Rawrarr could replace Prowlarr's role while providing broader media automation. Existing documentation described…
- [Run Rust Arr locally](2026-10-17-run-rust-arr-locally.md) — Compile and start the Rust Arr application as a local web server, then verify its HTTP routes. The code is committed on main; system Rust…
- [Rust Arr skeleton](2026-10-17-rust-arr-skeleton.md) — Start a new memory-safe Rust service skeleton in this repository, prepared for local development and Docker deployment. The repository was clean…
- [Require age identity backup confirmation](2026-10-18-rawrarr-age-backup-confirmation.md) — The user wants setup to require acknowledgement that the private age identity is saved somewhere recoverable before vault unlock. Previously, the…
- [Derive Rawrarr's theme from the age identity](2026-10-18-rawrarr-age-derived-theme.md) — The age setup screen already showed a colored fingerprint derived from the public age recipient, but the Rawrarr workspace kept a fixed green…
- [Keep private age identity off the setup screen](2026-10-18-rawrarr-age-identity-copy-screen.md) — The owner reported that newly generated private age identity text was displayed on the setup screen and requested a copy-only flow plus a visual…
- [Show a shortened age identity preview](2026-10-18-rawrarr-age-identity-short-preview.md) — Show a recognizable shortened form after generating an age identity while keeping the full private identity available only through the explicit…
- [Refresh the running age setup page](2026-10-18-rawrarr-age-preview-runtime-refresh.md) — The user reported that the age setup screen still showed the complete private identity despite the source change for a shortened preview. The…
- [Add color to the public age identity fingerprint](2026-10-18-rawrarr-age-public-fingerprint-colors.md) — The age setup page generated a symmetric square fingerprint for a public age recipient, but every square used the same green fill. Add a…
- [Clarify age identity setup actions](2026-10-18-rawrarr-age-setup-button-clarity.md) — The user created an age identity but could not distinguish the file-selection action from the vault-unlock action. In the UI, “Import from file”…
- [Keep the relay blind to subscriber app data](2026-10-18-rawrarr-blind-relay-age-boundary.md) — Record the trust boundary for the planned Rawrarr relay/subscription service. The user clarified that “main server” refers to this future cloud…
- [Plan Rawrarr cloud access](2026-10-18-rawrarr-cloud-access-architecture.md) — Record the user's architecture analogy: a Rawrarr cloud portal should help a person reach their own nodes while the hosted service itself has no…
- [Generated Docker Compose and node claim flow](2026-10-18-rawrarr-compose-claim-flow.md) — Capture the planned self-service deployment flow: a private or subscribed user gets a Rawrarr Docker Compose YAML file from the site, starts it…
- [Establish Rawrarr data domains](2026-10-18-rawrarr-data-domains.md) — Define stable paths for media, encrypted metadata, Syncthing-shared app state, and node-local state. The current deployment already mounted the…
- [Create the Rawrarr product design guide](2026-10-18-rawrarr-design-guide.md) — The project had technical architecture, product direction, and a user guide, but no dedicated living record of how the app should behave and…
- [Create age-encrypted sidecars during library scans](2026-10-18-rawrarr-encrypted-library-sidecars.md) — Make a Rawrarr storage/scanner node write JSON metadata for discovered media into the Syncthing-shared Rawrarr folder. Existing scans only…
- [Rawrarr as a device-to-device file-sharing system](2026-10-18-rawrarr-general-file-sharing-direction.md) — Capture the user's product direction that Rawrarr extends the broader Synkpieces idea: sharing files between devices is the core, while Radarr and…
- [Plan local backups of Rawrarr shared data](2026-10-18-rawrarr-local-backups.md) — Record the requested backup behavior: copy the Rawrarr Syncthing shared folder to a local subscriber node, keep backups local to that node, and…
- [Define Rawrarr node types and SwarmSail role](2026-10-18-rawrarr-node-types-and-swarmsail.md) — Clarify the deployment roles after the user specified that the operator's main node should eventually run SwarmSail and coordinate subscriber…
- [Fix local Rawrarr passkey origin](2026-10-18-rawrarr-passkey-origin.md) — The local Rawrarr login returned “Passkey verification failed; try again.” The container was published on host port 8084, but its configured…
- [Consolidate Rawrarr technical and user documentation](2026-10-18-rawrarr-project-documentation.md) — Create a clear project entry point that tells readers what Rawrarr can do now, how to run the Docker prototype, and which product ideas remain…
- [Extract Rawrarr into its own repository](2026-10-18-rawrarr-repository-extraction.md) — Move the Rawrarr application out of the mixed arr-services Git repository into the sibling Codebox/rawrarr repository while keeping pre-Rawrarr…
- [Store compressed Rawrarr logs in shared app data](2026-10-18-rawrarr-shared-compressed-logs.md) — Write compressed Rawrarr operational logs under the existing Syncthing-shared /shared folder. The application previously printed startup and…
- [Plan a Rawrarr subscription model](2026-10-18-rawrarr-subscription-model.md) — Turn the existing direction—24-hour trial, self-hosted deployment, hosted checkout, and a signed license—into a reviewable product and technical…
- [Plan optional Rawrarr donations](2026-10-18-plan-optional-donations.md) — Define voluntary one-time donations with provider-hosted checkout, separate from subscriptions and subscriber entitlements.
- [Consider a private cloud shard for subscribers](2026-10-18-rawrarr-subscriber-cloud-shard-idea.md) — Record the future idea of provisioning each subscriber a private cloud node with a disk quota, with Vult as a provider to evaluate.
- [Define SwarmSail tailnet account boundary](2026-10-18-rawrarr-swarmsail-tailnet-account-boundary.md) — Capture the user's decisions about the planned SwarmSail integration after reviewing [node types](../docs/node-types.md) and [cloud access…
- [Plan optional Tailscale access for Rawrarr nodes](2026-10-18-rawrarr-tailscale-integration-plan.md) — Record the near-term plan for optional Tailscale connectivity, a tailnet-scoped stable Rawrarr service name, and full web/API service on…
- [Reuse the existing local Compose deployment](2026-10-18-reuse-existing-local-compose-deployment.md) — Validate the standalone checkout against the existing OrbStack project, then restore its web container only after confirming the existing paths, port, project, and volumes.
- [Paste an existing age identity during setup](2026-10-18-rawrarr-paste-age-identity.md) — Remove file selection from the setup flow and import an existing identity by pasting it into the masked field.
- [Make Rawrarr container logs verbose](2026-10-18-rawrarr-verbose-container-logs.md) — Log routine HTTP request summaries and safe scan counts to container stdout, with a quiet-mode environment option.
- [Move local Rawrarr shared data outside Codebox](2026-10-18-rawrarr-local-shared-data-location.md) — Bind `/shared` to a persistent, Finder-accessible directory in Application Support and preserve the old folder as a rollback copy.
- [Display rawrarr in lowercase](2026-10-18-rawrarr-lowercase-brand-copy.md) — Keep the product name lowercase in user-facing interface copy, errors, and logs, with a regression test for displayed text.
- [Copy only the private age identity](2026-10-18-rawrarr-copy-private-identity-only.md) — Make the clipboard button extract the private key line and omit the public-recipient comment from an age identity file.
- [Report Rawrarr container health](2026-10-18-rawrarr-docker-healthchecks.md) — Add an HTTP health check to the application image alongside PostgreSQL's existing readiness check.
- [Verify private age key copy](2026-10-18-rawrarr-private-key-copy-verification.md) — Gate vault unlock on a random three-character check from the copied key and backup confirmation.
- [Guide new ID creation in a pop-up](2026-10-18-rawrarr-identity-creation-wizard.md) — Guide generated ID creation through copy, acknowledgement, and a three-character check before automatically opening the main workspace.
- [Refresh Rawrarr project documentation](2026-10-18-rawrarr-documentation-refresh.md) — Update the README, architecture, and design docs with current capabilities and separate them from planned hosted and multi-node behavior.
- [Consolidate Rawrarr project status into the roadmap](2026-10-18-rawrarr-roadmap.md) — Create one prioritized completed/outstanding work list and replace duplicated status queues with links to it.
- [Show the rawrarr logo on setup](2026-10-18-rawrarr-setup-logo.md) — Reuse the existing lowercase rawrarr logo asset on the age identity setup page.
- [Serve the rawrarr logo as JPEG XL](2026-10-18-rawrarr-jpeg-xl-logo.md) — Add a lossless JPEG XL logo source for supported browsers, retain the PNG fallback, and record animation as future work.

- [Resolve stale public age recipient](2026-10-18-age-recipient-conflict.md) — Preserve an unused setup recipient, unblock local onboarding, and distinguish identity conflicts from storage errors.

- [Preserve the current page on refresh](2026-10-18-preserve-page-on-refresh.md) — Keep workspace navigation through reloads and browser history.

- [Preserve the selected node page on refresh](2026-10-18-preserve-coordinator-node-on-refresh.md) — Retain the selected metadata node and embedded page when refreshing the coordinator workspace.

- [Browser-reachable Radarr movie links](2026-10-18-radarr-movie-links.md) — Link posters and titles only after a browser reachability check.

- [Plan the Mac mini storage node](2026-10-18-mac-mini-storage-node-plan.md) — Plan OrbStack, read-only silo media access, and joining the current instance.

- [Movie quality profile cache](2026-10-18-movie-quality-profile-cache.md) — Resolve quality names from shared metadata, fetch missing Radarr profiles, and persist encrypted updates.

- [Responsive Arr library loading and refresh](2026-10-18-responsive-movies-refresh.md) — async artwork links, retained rows on refresh errors, bounded encrypted metadata reads, and memory-only snapshots on page reload.

- [Separate movie identifiers](2026-10-18-separate-movie-identifiers.md) — keep titles clean and display TMDb IDs in their own column.
- [Coordinator roles and storage device-code linking](2026-10-18-coordinator-storage-device-linking.md) — Define coordinator/storage/shard/service boundaries and plan Tailscale HTTPS with owner-approved device codes.

- [Plan two-Mac Syncthing replication](2026-10-18-plan-two-mac-syncthing.md) — Explicit shared-folder containers, peer setup, exclusions, and verification for coordinator/storage deployment.

- [MacBook Syncthing container](2026-10-18-macbook-syncthing-container.md) — Local review companion, actual shared-folder scan, localhost GUI, and no remote peers.

- [Sonarr integration and shared library layout](2026-10-18-sonarr-shared-library.md): encrypted Sonarr connections, read-only series, and common Arr columns.
- [Finish Sonarr connection settings](2026-10-18-sonarr-settings-flow.md) — Implement the coordinator Sonarr URL/API-key form, age-encrypted persistence, and deploy the direct metadata route.

- [Route Rawrarr HTTPS to the coordinator](2026-10-18-coordinator-https-routing.md) — Correct the live Tailscale proxy target and verify coordinator-only HTTPS routing.

- [Per-poster request timing](2026-10-18-per-poster-request-timing.md) — Exclude scrolling and browser queue waits from artwork durations.

- [Fix coordinator health 502](2026-10-18-fix-coordinator-health-502.md) — Correct Tailscale Serve port 443 to the coordinator proxy.

- [Temporarily disable age character verification](2026-10-18-disable-age-character-check.md) — Require copy and saved-ID acknowledgement, defer character verification, and refresh the local app.

- [Keep the public coordinator URL on the coordinator](2026-10-18-coordinator-url-recurrence.md) — Repair Serve drift and record a required route read-back and HTTPS health check.

- [Add a coordinator-origin passkey to the existing account](2026-10-18-add-coordinator-origin-passkey.md) — Allow an owner signed in on a metadata node to register a passkey for the coordinator's HTTPS origin and attach it to the same PostgreSQL account. WebAuthn credentials are scoped to their relying-party origin;

- [Explain age challenge validation state](2026-10-18-age-challenge-validation-feedback.md) — When a user enters three characters but Continue remains disabled, show whether the answer is incomplete, incorrect, or verified, and remind them which whole-key positions to

- [Move age ID import into a pop-up](2026-10-18-age-id-import-popup.md) — Change the existing identity action to Import ID and move the masked private-key field off the setup page into a dialog opened by that action. Previously the paste field was always visible beside ID creation.

- [Improve age setup dialog emphasis](2026-10-18-age-setup-dialog-emphasis.md) — Make the private identity recovery warning easy to notice and visually separate the manual three-character key check so users understand that they must consult their saved

- [Count saved artwork across the library](2026-10-18-artwork-progress-counts.md) — Correct the Movies progress bar after the user observed 900+ encrypted poster files in shared metadata while the bar showed only

- [Automatically backfill missing movie artwork](2026-10-18-automatic-movie-artwork-backfill.md) — Check whether Rawrarr had movie posters that were not yet saved in shared metadata, backfill them, and make future backfills automatic. The local metadata service was healthy at http://127.0.0.1:7297. Shared

- [Central coordinator, storage, and metadata containers](2026-10-18-central-coordinator-storage-metadata.md) — Implement the user's revised three-role model and prepare Mac mini installation. Coordinator is central passkey/authentication and node coordination only; storage and metadata both serve the user app, Radarr,

- [Clarify age key challenge character counting](2026-10-18-clarify-age-key-character-count.md) — Help users who receive a mismatch understand how to count the requested characters in a generated post-quantum age

- [Coordinator-owned health page](2026-10-18-coordinator-health-page.md) — Give the coordinator its own styled health page, matching the metadata status design without forwarding to subscriber nodes. HTTPS already routes to coordinator; /health returns plain text and / returns 401.

- [Plan the coordinator node workspace and page relay](2026-10-18-coordinator-node-workspace-relay-plan.md) — Make https://rawrarr.risk-mermaid.ts.net/ the signed-in Rawrarr entry point. It should show the existing coordinator and connected metadata/storage nodes, accept pending node enrollment, and display a selected

- [Deploy the age ID refresh fix locally](2026-10-18-deploy-age-id-refresh-fix.md) — Apply the camel-case age identity request decoding fix to the persistent local Rawrarr service. The app was healthy at 127.0.0.1:7297, and the rawrarr-local Compose project had separate persistent app data,

- [Document the current age identity dialog flow](2026-10-18-document-age-dialog-flow.md) — Align the user guide, design guide, architecture, and roadmap with the implemented age identity copy and verification

- [Add an explicit Continue action to age setup](2026-10-18-explicit-age-setup-continue.md) — After creating and copying an age ID, let the user enter the requested key characters and explicitly continue. Keep the action disabled until the entered characters

- [Fix stale browser module on coordinator passkey page](2026-10-18-fix-stale-coordinator-passkey-module.md) — The coordinator passkey page remained at its initial status. The browser console showed that coordinator-passkey.mjs imported encodeCredential from a cached older /assets/passkey-client.mjs that did not export

- [Group metadata by media type](2026-10-18-group-metadata-by-media-type.md) — Put media-specific metadata under media-type directories, with separate text and artwork subdirectories. The existing layout put those kinds first (metadata/text/movies and metadata/artwork/movies); the shared

- [Group the MacBook Tailscale container with rawrarr-local](2026-10-18-group-tailscale-in-rawrarr-local.md) — Place the MacBook Tailscale container in the existing rawrarr-local Docker Compose project group without restarting Rawrarr services or losing the pending Tailscale identity

- [Improve the Rawrarr health page](2026-10-18-health-page.md) — Replace the plain ok response at /health with a clear, styled status

- [Implement browser mini metadata export](2026-10-18-implement-browser-mini-metadata-export.md) — Implement a testable browser proof of concept that exports a small copy of encrypted movie metadata and matching artwork. The existing service keeps the age identity and decrypted config in its own unlocked

- [Increase Radarr movie table text size](2026-10-18-larger-movie-table-text.md) — Make the small text in the Movies library table easier to read. Table body cells used a 10px font

- [Keep a local Radarr settings mirror beside encrypted shared config](2026-10-18-local-radarr-config-mirror.md) — Let local tools inspect the configured Radarr connection without passkey or age handling while keeping production settings encrypted. Rawrarr already stored settings in /shared/config.aged; local /data held

- [Use lowercase Rawrarr Application Support directory](2026-10-18-lowercase-application-support-directory.md) — Use ~/Library/Application Support/rawrarr as the canonical local Rawrarr data root. The persistent Compose service mounted ~/Library/Application Support/Rawrarr/shared-data; because the host filesystem is

- [Deploy the Mac mini storage node](2026-10-18-mac-mini-storage-node-deployment.md) — Connect a Mac mini storage node and the seeded MacBook metadata node to the already provisioned, healthy coordinator. Use containerized Tailscale and Syncthing, and require explicit owner approval for the

- [Mixed and distributed media libraries](2026-10-18-mixed-and-distributed-media-libraries.md) — Clarify how storage nodes represent libraries when one machine holds several media kinds or when media kinds live on different machines. Existing architecture already allowed multiple named libraries per

- [Keep Movies summary cards on one row](2026-10-18-movies-summary-one-row.md) — Place the four Movies summary cards, including Shared metadata, together on one

- [Opaque movie metadata filenames](2026-10-18-opaque-movie-metadata-filenames.md) — Replace numbered movie metadata and artwork filenames with HMAC-derived names while retaining decryptable ciphertext and restart the sequential artwork scraper on the metadata node. The existing encrypted

- [Simplify passkey sign-in for returning users](2026-10-18-passkey-signin-existing-account.md) — Remove the recurring username prompt from passkey sign-in. PostgreSQL already stores the username/display name and determines whether first-user setup is complete, but the page showed a username field whenever

- [Unlock an existing pasted age ID directly](2026-10-18-pasted-age-id-direct-unlock.md) — Fix the existing-ID onboarding path, which sent pasted IDs through the new-ID copy and backup dialog and could look as if Rawrarr still expected ID creation. Related context: remember the age ID locally and

- [Plan browser-side metadata export](2026-10-18-plan-browser-metadata-export.md) — Record a safe product and architecture plan for a user-downloadable plaintext copy of Rawrarr's metadata tree. The current shared tree contains age-encrypted metadata and artwork, with movie filenames derived

- [Plan the first production deployment](2026-10-18-production-deployment-plan.md) — Record a practical first-production database choice and deployment checklist, including a later managed Postgres migration

- [Show Radarr artwork load time](2026-10-18-radarr-artwork-load-time.md) — Add a browser-observed duration beside each Radarr poster so the page can be used to inspect the time spent fetching shared age-encrypted artwork. The table already showed artwork source; poster elements are

- [Connect Radarr before saving encrypted credentials](2026-10-18-radarr-connection-save.md) — Ensure a user can enter a Radarr URL and API key, confirm that the endpoint accepts them, and save them to the synchronized age-encrypted configuration. The UI already provided the form and PUT

- [Show and share Radarr movie artwork](2026-10-18-radarr-movie-artwork.md) — Display Radarr posters in Rawrarr and begin storing Radarr-derived movie metadata in the shared metadata domain. The Movies view previously showed a text table. Shared age-encrypted artwork existed under

- [Plan Rawrarr container distribution](2026-10-18-rawrarr-container-distribution.md) — Capture an eventual path for distributing the Rawrarr Docker container to self-hosted subscribers. The repository currently has a development Compose file that builds rawrarr:local; it has no published image

- [Reserve localhost port 7297 for rawrarr-local](2026-10-18-rawrarr-local-port-7297.md) — Use localhost port 7297 for the persistent rawrarr-local web service and treat that port as reserved for Rawrarr in the local

- [Remember the age ID on this device](2026-10-18-remember-age-id-local.md) — Reduce repeated age identity entry during local Rawrarr testing. Previously the unlocked vault existed only in process memory, so a restart or logout cleared it. Related context: age-encrypted config and

- [Remember the age ID after refresh](2026-10-18-remember-age-id-refresh.md) — Fix the setup flow so selecting Keep this ID on this device persists the age identity for later app starts. The browser posts rememberOnDevice, but the Rust request type used rememberondevice without

- [Remove the age setup dialog close action](2026-10-18-remove-age-dialog-close-action.md) — Keep the identity-copy dialog focused on completing the copy and verification flow; do not offer a Close button that can leave the user without a forward

- [Require HTTPS for Arr connections](2026-10-18-require-https-for-arr-connections.md) — Require encrypted HTTPS transport whenever Rawrarr connects to Radarr, Sonarr, and other Arr services. The shared Arr client previously accepted both HTTP and

- [Dedicated shared filename secret](2026-10-18-shared-filename-secret.md) — Add a shared secret exclusively for opaque metadata filename derivation in the main age-encrypted config. Existing files keep their current names; HMAC naming and migration are a separate step. The local

- [Show movie artwork source](2026-10-18-show-movie-artwork-source.md) — Show whether each poster in the Radarr Movies table is loaded from Rawrarr's shared encrypted artwork cache or will be loaded from Radarr. The table displayed posters but gave no source status. Shared artwork

- [Show shared metadata size on Movies](2026-10-18-show-shared-metadata-size.md) — Show the current size of the encrypted shared metadata tree on the Movies page and keep it current while the metadata backfill runs. The page already polled artwork progress every three seconds, but did not

- [Show media and total metadata size](2026-10-18-show-media-metadata-size.md) — Replace the artwork summary on Movies and add a TV summary showing media-subtree and total metadata storage.

- [Start the MacBook Tailscale container](2026-10-18-start-macbook-tailscale-container.md) — Start a dedicated Tailscale container on the MacBook Pro and present Tailscale's interactive login flow so the owner can add this container as a tailnet node. Continue with Serve only after

- [Test Radarr metadata scrape](2026-10-18-test-radarr-metadata-scrape.md) — Scrape a few movie records and posters through the local Rawrarr app so the user can inspect the resulting shared files. The app was healthy on 127.0.0.1:7297; the host /shared bind resolved to

- [Plan Tailscale containers on both Macs](2026-10-18-two-mac-tailscale-containers.md) — Plan separate Tailscale containers on the MacBook Pro and Mac mini, with the MacBook container using Tailscale Serve for the Rawrarr coordinator. No tailnet or container configuration is changed in this

- [Compact completed artwork cache status](2026-10-18-compact-completed-artwork-status.md) — Hide the artwork progress bar and detailed saved count after the cache is complete, leaving a compact confirmation on Movies.

- [Keep metadata independent of the coordinator service](2026-10-18-metadata-external-coordinator.md) — Configure metadata nodes to reach a coordinator through an HTTPS URL without a Compose service dependency.

- [Name Rawrarr node types](2026-10-18-node-type-names.md) — Historical naming decision, superseded by the 2026-10-20 fossil/boneyard mapping.

- [Propose signed device identities and role claims](2026-10-18-device-signing-identity-proposal.md) — Separate future cryptographic device IDs from display names and node functions, while preserving existing deployment IDs until migration is designed.

- [Implement per-instance device signing](2026-10-18-implement-device-signing.md) — Add owner-rooted ML-DSA device enrollment and a backward-compatible migration for the deployed token-based nodes.

- [Remember age ID in browser](2026-10-18-remember-age-id-in-browser.md) — optional browser-local age ID retention with a highest-security warning and sign-out cleanup.
- [Manage browser age ID storage in Settings](2026-10-18-age-id-storage-settings.md) — Show browser retention state and validate, save, or remove the age private ID without persisting root signing material.
- [Manage Radarr and Sonarr from Settings](2026-10-18-arr-connections-in-settings.md) — Add formatted Settings forms and saved-connection confirmation boxes for both Arr services.

- [Plan browser security tests](2026-10-18-plan-browser-security-tests.md) — Plan Playwright integration scenarios for metadata and storage nodes, coordinator plaintext-leak assertions, and security claim limits.

- [Fix Movies age-ID state](2026-10-18-fix-movies-age-id-state.md) — Rehydrate a browser-saved age ID when opening Movies and avoid presenting request/decryption failures as a missing-ID prompt.

- [Align Devices and Movies route status](2026-10-18-align-device-and-movies-route-status.md) — Make Devices label a node Direct only after the browser's direct health probe succeeds, matching Movies route selection.

- [Share the device route status checker](2026-10-18-share-device-route-status-checker.md) — Extract a reusable all-device Direct, Relay, or Offline check and use it across Devices, Movies, and TV.

- [Place Settings profile and age ID side by side](2026-10-18-place-settings-profile-and-age-id-side-by-side.md) — Put the profile and browser age private ID controls in a responsive shared row.

- [Move route latency to Devices](2026-10-18-move-route-latency-to-devices.md) — Show measured direct Tailscale latency on device cards and remove route-status banners from Movies and TV.
- [Overview genre pie charts](2026-10-18-overview-genre-pie-charts.md) — Show larger Radarr and Sonarr genre pies with up to ten named genres and an aggregated Other slice.
- [Give the profile name room for a two by two layout](2026-10-18-settings-profile-words-layout.md) — Narrow the browser Age ID card and display profile words in two columns beside the avatar.
- [Fix Overview genre data](2026-10-18-fix-overview-genre-data.md) — Preserve Radarr genres in the browser library payload and initialize Arr loaders for Overview charts.
- [Task-owned Git workflow](2026-10-18-task-owned-git-workflow.md) — Allow focused task-owned commits as work progresses and use dedicated worktrees when concurrent edits overlap.
- [Review current Rawrarr documentation](2026-10-18-current-state-documentation-review.md) — Align setup, architecture, export, scanning, and roadmap claims with current source and observed local port routing.
- [Record documentation review commits](2026-10-18-documentation-commit-history.md) — Save the documentation-review commit in project instructions and require a separate history update after future reviews.
