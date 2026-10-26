import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [coordinator, auth, nodeAuth, workspace, workspaceScript, deviceRoutes, metadataUi, compose, metadataExport, mainRs, tailscaleCompose, boneyardCompose, coordinatorAvatar, internalMiniCompose] = await Promise.all([
  read('rust-arr/src/coordinator.rs'),
  read('rust-arr/src/auth.rs'),
  read('rust-arr/src/node_auth.rs'),
  read('rust-arr/src/coordinator_workspace.html'),
  read('rust-arr/src/coordinator-workspace.mjs'),
  read('rust-arr/src/coordinator-device-routes.mjs'),
  read('rust-arr/src/ui.html'),
  read('docker-compose.yml'),
  read('rust-arr/web-export/metadata-export.js'),
  read('rust-arr/src/main.rs'),
  read('docker-compose.tailscale-macbook.yml'),
  read('docker-compose.storage.yml'),
  read('rust-arr/src/coordinator_avatar.rs'),
  read('docker-compose.internal-mini.yml'),
]);

test('coordinator labels the Sonarr section TV in navigation and its page heading', () => {
  assert.doesNotMatch(workspace, /id="tv-route-status" class="route-status"/);
  assert.match(workspace, /data-view=\"series\"[^>]*>[\s\S]*?<span class=\"nav-text\">TV<\/span>/);
  assert.match(workspace, /id=\"series-view\" hidden><h1 class=\"page-title\">TV<\/h1>[\s\S]*?id=\"sonarr-settings\"/);
  assert.match(workspace, /id=\"movies-view\" hidden><h1 class=\"page-title\">Movies<\/h1>[\s\S]*?id=\"radarr-settings\"/);
  assert.match(workspace, /id=\"radarr-connection-card\" class=\"connection-card\"[^>]* hidden/);
  assert.match(workspace, /id=\"sonarr-connection-card\" class=\"connection-card\"[^>]* hidden/);
  assert.match(workspace, /id=\"tv-library\" hidden[^>]*aria-label=\"Sonarr series library\"/);
  assert.match(workspace, /id=\"sonarr-series\"/);
  assert.match(workspace, /id=\"refresh-series\" type=\"button\"/);
  assert.match(workspaceScript, /arrDirectRequest\('\/sonarr\/series'\)/);
  assert.match(workspaceScript, /setArrSummaryValues\('sonarr',\s*\{[\s\S]*?filesPresent: String\(episodesPresent\)/);
  assert.match(workspaceScript, /document\.getElementById\(fields\.mediaCard\)\.hidden = Boolean\(settings\.configured\)/);
  assert.match(workspaceScript, /document\.getElementById\(fields\.mediaCard\)\.hidden = false/);
  assert.match(workspace, /class=\"connection-card\"/);
  assert.match(workspace, /class=\"connection-form\"/);
  assert.match(workspace, /id=\"settings-radarr-settings\" class=\"connection-form\"/);
  assert.match(workspace, /id=\"settings-sonarr-settings\" class=\"connection-form\"/);
  assert.match(workspace, /id=\"settings-radarr-saved\" class=\"connection-saved\" hidden/);
  assert.match(workspace, /id=\"settings-sonarr-saved\" class=\"connection-saved\" hidden/);
  assert.match(workspace, /\.connection-form input \{ width:100%; min-height:42px/);
  assert.match(workspaceScript, /name === 'tv' \? 'series' : name/);
  assert.match(workspaceScript, /view === 'series' \? 'tv' : view/);
  assert.match(workspaceScript, /if \(view === 'movies' && updateHash\)[\s\S]*?updateAgeIdControls\(\)[\s\S]*?if \(ageIdentityInMemory && selectedNode\) void unlockMetadata\(\)/);
  assert.match(workspaceScript, /catch \(error\) \{[\s\S]*?moviesPanel\.replaceChildren\(\);\s*updateAgeIdControls\(\);\s*contentStatus\.textContent/);
});

test('Movies and TV cached library data stays hidden until an age ID is loaded', () => {
  assert.match(workspaceScript, /async function unlockMetadata\(\) \{[\s\S]*?if \(!selectedNode \|\| !identity\) return;/);
  assert.match(workspaceScript, /if \(view === 'movies' && updateHash\) \{[\s\S]*?if \(ageIdentityInMemory && selectedNode\) void unlockMetadata\(\)/);
  assert.match(workspaceScript, /if \(view === 'series' && updateHash\) \{\s*const cachedSeries = ageIdentityInMemory \? readLibraryTextCache\(selectedNode, 'tv'\) : null;/);
});

test('pending-node approval dialog is outside the hidden Settings section', () => {
  const settingsSection = workspace.match(/<section id="settings-view" hidden>[\s\S]*?<\/section>/)?.[0];
  assert.ok(settingsSection, 'Settings section exists');
  assert.doesNotMatch(settingsSection, /id="approve-node-dialog"/);
  assert.match(workspace, /<\/section>\s*<dialog id="approve-node-dialog"/);
  assert.match(workspaceScript, /approveNodeDialog\.showModal\(\)/);
});

test('device shared-folder health is concise and formatted without exposing the folder ID', () => {
  assert.match(workspace, /\.device-sync-health \{ display:flex; flex-wrap:wrap; align-items:center; gap:4px/);
  assert.match(workspaceScript, /const setSyncHealth = parts =>/);
  assert.match(workspaceScript, /parts\.push\(`\$\{peers\.filter\(peer => peer\.connected\)\.length\}\/\$\{peers\.length\} peers connected`\)/);
  assert.match(workspaceScript, /`\$\{peer\.folderConnectedPeerCount\}\/\$\{peer\.folderPeerCount\} peers connected`/);
  assert.doesNotMatch(workspaceScript, /return `\$\{folder\.folderId\} ·/);
  assert.doesNotMatch(workspaceScript, /\$\{peer\.folderId\} ·/);
});

test('Devices shows a network diagram with one link for each device pair', () => {
  assert.match(workspace, /<section id="device-topology" class="topology-section" aria-labelledby="device-topology-title" hidden>[\s\S]*?<h2 id="device-topology-title">Topology<\/h2>/);
  assert.match(workspace, /<div id="device-topology-content"><\/div>/);
  assert.match(workspaceScript, /function renderDeviceTopology\(nodes, observedPeers, deviceNames\)/);
  assert.match(workspaceScript, /deviceNames\[node\.nodeId\] \|\| node\.displayName \|\| node\.nodeId/);
  assert.match(workspaceScript, /deviceNameTokens\(node\.displayName\)/);
  assert.match(workspaceScript, /getElementById\('device-topology-content'\)/);
  assert.match(workspaceScript, /nodes\.length \* \(nodes\.length - 1\) \/ 2/);
  assert.match(workspaceScript, /for \(let leftIndex = 0; leftIndex < count; leftIndex \+= 1\)/);
  assert.match(workspaceScript, /for \(let rightIndex = leftIndex \+ 1; rightIndex < count; rightIndex \+= 1\)/);
  assert.match(workspaceScript, /topology-edge\$\{observed \? observed\.connected \? ' connected' : ' disconnected' : ''\}/);
  assert.match(workspaceScript, /Add another device to see its links in the network diagram\./);
  assert.match(workspace, /\.topology-edge\.connected \{ stroke:var\(--green\); stroke-width:3 \}/);
  assert.match(workspace, /\.topology-edge\.disconnected \{ stroke:#ef7777 \}/);
});

test('Devices keeps topology hidden until an enrolled device is online', () => {
  assert.match(workspaceScript, /const topologySection = document\.getElementById\('device-topology'\);\s*emptyDevices\.hidden = nodes\.length !== 0;\s*topologySection\.hidden = !nodes\.some\(node => node\.online === true\);/);
  assert.match(workspaceScript, /if \(nodes\.some\(node => node\.online === true\)\) \{\s*renderDeviceTopology\(nodes, observedSyncthingPeers, deviceNames\);\s*\}/);
  assert.match(workspace, /<section id="device-topology"[^>]*hidden/);
});

test('device names have an adjacent Edit button that unlocks the name field', () => {
  assert.match(workspaceScript, /nameInput\.readOnly = true/);
  assert.match(workspaceScript, /editNameButton\.textContent = 'Edit'/);
  assert.match(workspaceScript, /nameGroup\.append\(nameInput, editNameButton\)/);
  assert.match(workspaceScript, /nameInput\.readOnly = false;\s*nameInput\.focus\(\);/);
  assert.match(workspaceScript, /saveDeviceName\(ageIdentity/);
  assert.match(workspace, /\.device-name-group \{ display:flex/);
});

test('metadata export in Settings prompts for an age ID when tab memory is empty', () => {
  assert.match(workspace, /id="metadata-export-dialog"/);
  assert.match(workspace, /id="metadata-export-identity" type="password" autocomplete="off"/);
  assert.match(workspace, /After a successful export it is saved in this browser\. It is never sent to roarlay/);
  assert.match(workspaceScript, /if \(ageIdentityInMemory\)[\s\S]*?runMetadataExport\(ageIdentityInMemory\)/);
  assert.match(workspaceScript, /metadataExportDialog\.showModal\(\)/);
  assert.match(workspaceScript, /const identity = metadataExportInput\.value\.trim\(\)/);
  assert.match(workspaceScript, /metadataExportInput\.value = ''/);
  assert.doesNotMatch(workspaceScript, /Unlock your age private ID in Movies before exporting metadata/);
});

test('Radarr and Sonarr settings use the configured user-node URL directly from the browser', () => {
  assert.match(workspace, /id=\"sonarr-settings\"/);
  assert.match(workspace, /id=\"sonarr-url\"/);
  assert.match(workspace, /id=\"sonarr-key\"[^>]*type=\"password\"/);
  assert.match(workspace, /roarlay never receives the API key/);
  assert.match(workspaceScript, /async function arrDirectUrl\(\)/);
  assert.match(workspaceScript, /async function arrDirectUrl\(\)[\s\S]*?const route = await chooseNodeRoute\(node, true\)/);
  assert.match(workspaceScript, /return `\$\{route\.base\}\/api`/);
  assert.match(workspaceScript, /const canOpenNode = node => node\.relayAvailable \|\| Boolean\(node\.directUrl\)/);
  assert.match(workspaceScript, /fetch\(`\$\{base\}\$\{path\}`/);
  assert.match(workspaceScript, /body: JSON\.stringify\(\{ identity, rememberOnDevice: false \}\)/);
  assert.match(workspaceScript, /apiKey: document\.getElementById\(copy\.key\)\.value/);
  assert.match(workspaceScript, /uiUsername: document\.getElementById\(copy\.uiUsername\)\.value/);
  assert.match(mainRs, /\.route\("\/api\/settings\/age", get\(get_age_settings\)\.post\(unlock_age_settings\)\)/);
  assert.match(mainRs, /\.route\(\s*"\/api\/settings\/sonarr",[\s\S]*?\.delete\(sonarr::clear\)/);
  assert.match(mainRs, /let direct_protected = Router::new\(\)[\s\S]*?\.route\("\/api\/sonarr\/status", get\(sonarr::status\)\)[\s\S]*?\.route\("\/api\/sonarr\/series", get\(sonarr::series\)\)/);
  assert.match(mainRs, /\.route\(\s*"\/api\/settings\/radarr",[\s\S]*?\.delete\(delete_radarr_settings\)/);
  assert.doesNotMatch(coordinator, /"api\/settings\/sonarr"/);
});

test('encrypted metadata fetches include the direct node browser session cookie', () => {
  assert.match(metadataExport, /async function fetchBytes\(url\)[\s\S]*?credentials: 'include'/);
});

test('coordinator serves the profile-color module imported by its workspace script', () => {
  assert.match(workspaceScript, /from '\.\/profile-color\.mjs'/);
  assert.match(coordinator, /\.route\("\/assets\/profile-color\.mjs", get\(coordinator_profile_color_script\)\)/);
  assert.match(coordinator, /async fn coordinator_profile_color_script\(\)[\s\S]*?include_str!\("profile-color\.mjs"\)/);
});

test('coordinator root selects passkey sign-in or an owner workspace while health remains public', () => {
  assert.match(coordinator, /\.route\("\/", get\(coordinator_home\)\)/);
  assert.match(coordinator, /\.route\("\/health", get\(coordinator_health\)\)/);
  assert.match(coordinator, /is_authenticated\(&state\.coordinator_auth, &headers\)/);
  assert.match(auth, /next_path: &'static str/);
  assert.match(coordinator, /next_path: "\/"/);
  assert.match(auth, /auth\.node_id == "coordinator" \{\s*"\/setup\/node"/);
  assert.match(workspace, /id="age-id"/);
  assert.match(workspace, /id="age-id-entry"/);
  assert.doesNotMatch(workspace, /id="remember-age-id"/);
  assert.match(workspace, /Your ID is saved in this browser so you can use the main pages without entering it again/);
  assert.match(workspace, /id="age-storage-title">Age private ID in this browser/);
  assert.match(workspace, /id="age-storage-status"/);
  assert.match(workspace, /id="save-age-id"/);
  assert.match(workspace, /id="remove-age-id"/);
  assert.match(workspace, /User signing keys are never saved in browser storage/);
  assert.match(workspace, /id="clear-key" type="button" hidden/);
  assert.match(workspace, /<title>rawrarr<\/title>/);
  assert.match(workspace, /saved in this browser/);
  assert.match(workspaceScript, /let ageIdentityInMemory = rememberedAgeIdentity/);
  assert.match(workspaceScript, /ageIdentityInMemory = identity/);
  assert.match(workspaceScript, /addEventListener\('pagehide', \(\) =>/);
  assert.match(workspaceScript, /localStorage\.getItem\(ageSessionKey\)/);
  assert.match(workspaceScript, /localStorage\.setItem\(ageSessionKey, identity\)/);
  assert.match(workspaceScript, /localStorage\.removeItem\(ageSessionKey\)/);
  assert.match(workspaceScript, /arrDirectRequest\('\/settings\/age'/);
  assert.doesNotMatch(workspaceScript, /localStorage\.(?:getItem|setItem)\([^)]*(?:root|signing)/i);
  assert.match(workspaceScript, /document\.getElementById\('sign-out'\)\.addEventListener[\s\S]*?clearAgeIdentity\(\)/);
  assert.doesNotMatch(workspaceScript, /sessionStorage\.(?:getItem|setItem)\(ageSessionKey/);
  assert.match(workspaceScript, /ageIdEntry\.hidden = hasIdentity/);
  assert.match(workspaceScript, /clearKeyButton\.hidden = !hasIdentity/);
  assert.match(workspaceScript, /updateAgeIdControls\(\);/);
  assert.doesNotMatch(workspace, /<iframe/i);
  assert.match(workspaceScript, /\/api\/workspace\/nodes/);
});

test('device approvals use a volatile browser form and root-signed exact node request', async () => {
  const [onboarding, onboardingScript, signingClient, subscriberComposeSource] = await Promise.all([
    read('rust-arr/src/subscriber_onboarding.html'),
    read('rust-arr/web-export/subscriber-onboarding.js'),
    read('rust-arr/web-export/device-signing-client.js'),
    read('rust-arr/web-export/subscriber-compose.js'),
  ]);
  assert.match(workspace, /id="approve-node-age-id" type="password"/);
  assert.match(workspace, /id="approve-node-root-backup"/);
  assert.match(workspace, /id="approve-node-root-fingerprint"/);
  assert.match(workspaceScript, /signDeviceApproval\(encryptedBackup, ageIdentity, savedFingerprint/);
  assert.match(workspaceScript, /signedPayload: approval\.signedPayload/);
  assert.match(workspaceScript, /approveNodeDialog\.addEventListener\('close'/);
  assert.match(workspaceScript, /approveNodeDialog\.close\(\)/);
  assert.match(signingClient, /secretKey\?\.fill\(0\)/);
  assert.match(onboarding, /id="root-fingerprint-confirm"/);
  assert.match(onboarding, /Re-enter the root fingerprint you saved separately/);
  assert.match(onboardingScript, /rootFingerprint: pinnedFingerprint/);
  assert.match(subscriberComposeSource, /RAWARR_INSTANCE_ROOT_FINGERPRINT/);
  assert.match(onboardingScript, /pagehide/);
  assert.doesNotMatch(onboardingScript, /localStorage|sessionStorage/);
});

test('signed-in profile requests a coordinator-rendered JPEG XL dinosaur from the public age ID', () => {
  assert.match(coordinator, /\.route\("\/api\/workspace\/profile", get\(workspace_profile\)\)/);
  assert.match(coordinator, /SELECT id FROM rawrarr_users WHERE id=\$1/);
  assert.match(coordinator, /display_name: crate::auth::generated_account_name\(user_id\)/);
  assert.match(coordinator, /account_id: user_id/);
  assert.match(workspace, /id="profile-picture"/);
  assert.doesNotMatch(workspace, /Loading the public age ID from a fossil node/);
  assert.doesNotMatch(workspace, /Your name from passkey setup/);
  assert.match(workspace, /Generated dinosaur profile picture/);
  assert.match(workspace, /id="profile-name"/);
  assert.match(workspace, /id="sidebar-profile-name"/);
  assert.match(workspace, /id="sidebar-profile-picture"/);
  assert.doesNotMatch(workspace, /Discovery, libraries, and fulfillment|Early access · Radarr first/);
  assert.match(workspaceScript, /jsonRequest\('\/api\/workspace\/profile'\)/);
  assert.match(workspaceScript, /sidebarName\.replaceChildren\(\.\.\.words\.map/);
  assert.match(workspaceScript, /sidebarName\.setAttribute\('aria-label', words\.join\(' '\)\)/);
  assert.match(workspaceScript, /const words = \[dinosaurMainColor\(profileAvatarSeed\), dinosaurWingWord\(profileAvatarSeed\), baseFirst, baseSecond\]/);
  assert.match(workspaceScript, /words\.slice\(0, 2\)\.join\(' '\)/);
  assert.match(workspaceScript, /words\.slice\(0, 2\)\.join\(' '\)/);
  assert.match(workspaceScript, /words\.slice\(2\)\.join\(' '\)/);
  assert.match(workspaceScript, /sidebar-profile-picture-jxl'\)\.srcset = jxlUrl/);
  assert.match(workspace, /\.profile-avatar \{ display:block; overflow:hidden;/);
  assert.match(workspaceScript, /profileAvatarSeed = Array\.from\(digest, byte => byte\.toString\(16\)\.padStart\(2, '0'\)\)\.join\(''\)/);
  assert.match(workspaceScript, /profileAccountId = profile\.accountId \|\| ''/);
  assert.match(workspaceScript, /rawrarr-profile:\$\{profileAccountId \|\| profileBaseName\}/);
  assert.match(coordinator, /\.route\("\/api\/workspace\/profile-picture\/\{seed\}\/jxl", get\(workspace_profile_picture_jxl\)\)/);
  assert.match(coordinator, /\.route\("\/api\/workspace\/profile-picture\/\{seed\}\/png", get\(workspace_profile_picture_png\)\)/);
  assert.match(coordinator, /async fn workspace_profile_picture_jxl\([\s\S]*?authenticated_user_id\(&auth, &headers\)/);
  assert.match(coordinator, /async fn workspace_profile_picture_png\([\s\S]*?authenticated_user_id\(&auth, &headers\)/);
  assert.match(coordinator, /workspace_profile_picture_response\(encoded, "image\/jxl"\)/);
  assert.match(coordinator, /workspace_profile_picture_response\(encoded, "image\/png"\)/);
  assert.match(workspace, /<source id="profile-picture-jxl" type="image\/jxl">/);
  assert.match(workspace, /\.profile-avatar \{ display:block;/);
  assert.match(workspaceScript, /profile-picture\/\$\{profileAvatarSeed\}\/jxl/);
  assert.match(workspaceScript, /profile-picture\/\$\{profileAvatarSeed\}\/png/);
  assert.match(workspaceScript, /ageIdentityInMemory \|\| localStorage\.getItem\(ageSessionKey\)/);
  assert.match(workspaceScript, /deriveProfileRecipient\(identity\)/);
  assert.doesNotMatch(workspaceScript, /fetch\(`\/nodes\/\$\{encodeURIComponent\(node\.nodeId\)\}\/api\/metadata\/export\/recipient/);
  assert.match(metadataExport, /identityToRecipient\(identity\.trim\(\)\)/);
  assert.doesNotMatch(workspaceScript, /function profilePictureData\(digest\)/);
  assert.match(workspaceScript, /deriveNodeAvatarSeed\(node\.nodeId\)/);
  assert.match(workspaceScript, /device-avatar/);
  assert.match(workspaceScript, /device-picture\/\$\{seed\}\/jxl/);
  assert.match(workspaceScript, /device-picture\/\$\{seed\}\/png/);
  assert.match(coordinator, /device-picture\/\{seed\}\/jxl/);
  assert.match(coordinator, /device-picture\/\{seed\}\/png/);
  assert.match(coordinator, /api\/metadata\/export\/node-recipient/);
  assert.match(coordinator, /coordinator-node-avatar\.mjs/);
  assert.match(mainRs, /api\/metadata\/export\/node-recipient/);
  assert.match(mainRs, /AgeVault::node_identity_recipient/);
  assert.match(mainRs, /AgeVault::ensure_node_identity\(\)/);
  assert.match(workspaceScript, /crypto\.subtle\.digest\('SHA-256'/);
  assert.match(coordinatorAvatar, /"rsvg-convert"/);
  assert.match(coordinatorAvatar, /"cjxl"/);
  assert.match(coordinatorAvatar, /fn svg_for_digest/);
  assert.match(coordinatorAvatar, /fn fossil_svg_for_digest/);
  assert.match(coordinatorAvatar, /fossil_device_art_is_deterministic_and_uses_a_shell_fossil/);
  assert.match(coordinatorAvatar, /if wings\.is_empty\(\)/);
  assert.match(coordinatorAvatar, /encodes_generated_avatar_as_a_jpeg_xl_codestream/);
  assert.match(coordinator, /private, max-age=31536000, immutable/);
  assert.match(workspaceScript, /Generated dinosaur profile picture/);
  assert.doesNotMatch(workspaceScript, /Generated in this browser from your public age ID\. No image is uploaded\./);
  assert.match(workspaceScript, /profile-avatar-status'\)\.textContent = ''/);
  assert.doesNotMatch(coordinator, /SELECT\s+(?:profile_picture|avatar_seed)/i);
});

test('device pictures use the deterministic fossil renderer while profile pictures retain dinosaurs', () => {
  assert.match(workspaceScript, /device-picture\/\$\{seed\}\/jxl/);
  assert.match(workspaceScript, /device-picture\/\$\{seed\}\/png/);
  assert.match(coordinator, /device-picture\/\{seed\}\/jxl/);
  assert.match(coordinator, /device-picture\/\{seed\}\/png/);
  assert.match(coordinatorAvatar, /pub fn render_fossil_jxl/);
  assert.match(coordinatorAvatar, /pub fn render_fossil_png/);
  assert.match(coordinatorAvatar, /fn fossil_svg_for_digest/);
  assert.match(coordinatorAvatar, /fossil_device_art_is_deterministic_and_uses_a_shell_fossil/);
  assert.match(workspaceScript, /profile-picture\/\$\{profileAvatarSeed\}\/jxl/);
  assert.match(workspaceScript, /profile-picture\/\$\{profileAvatarSeed\}\/png/);
});

test('sidebar profile name displays four words in two columns with an accessible full name', () => {
  assert.match(workspace, /\.sidebar-profile-name \{[^}]*display:grid;[^}]*grid-template-columns:max-content max-content/);
  assert.match(workspaceScript, /const words = \[dinosaurMainColor\(profileAvatarSeed\), dinosaurWingWord\(profileAvatarSeed\), baseFirst, baseSecond\]/);
  assert.match(workspaceScript, /sidebarName\.replaceChildren\(\.\.\.words\.map\(word =>/);
  assert.match(workspaceScript, /sidebarName\.setAttribute\('aria-label', words\.join\(' '\)\)/);
});

test('node list and relay are restricted to the signed-in owner and trusted configured targets', () => {
  assert.match(coordinator, /WHERE n\.user_id=\$1 ORDER BY CASE n\.role/);
  assert.match(coordinator, /WHERE id=\$1 AND user_id=\$2/);
  assert.match(coordinator, /RAWARR_NODE_RELAY_TARGETS/);
  assert.match(coordinator, /state\.relay_targets\.get\(&node_id\)/);
  assert.match(coordinator, /rawrarr_node_session/);
  assert.match(coordinator, /node_id=\$3 AND expires_at > now\(\)/);
  assert.match(coordinator, /fn rewrite_node_asset_paths/);
  assert.match(nodeAuth, /tokio::spawn\(async move/);
  assert.match(nodeAuth, /Duration::from_secs\(30\)/);
  assert.match(compose, /RAWARR_NODE_RELAY_TARGETS/);
});

test('internal coordinator maps the newly approved fossil ID to its relay and direct Tailscale routes', () => {
  assert.match(internalMiniCompose, /"device-d3ee53b1d561fb349955b199":"https:\/\/rawrarr-macbook-pro\.risk-mermaid\.ts\.net\/"/);
  assert.match(internalMiniCompose, /"device-d3ee53b1d561fb349955b199":"https:\/\/rawrarr-macbook-pro\.risk-mermaid\.ts\.net\/direct-api\/"/);
  assert.match(internalMiniCompose, /"device-9288eb3b039608d16a9102a5"/);
});

test('coordinator relay can reach tailnet nodes through its Tailscale SOCKS5 sidecar', () => {
  assert.match(coordinator, /RAWARR_NODE_RELAY_SOCKS5_URL/);
  assert.match(coordinator, /reqwest::Proxy::all\(&proxy_url\)/);
  assert.match(coordinator, /NoProxy::from_string\("fossil"\)/);
  assert.match(compose, /"mac-mini-storage":"https:\/\/rawrarr-mini\.risk-mermaid\.ts\.net"/);
  assert.match(compose, /RAWARR_NODE_RELAY_SOCKS5_URL/);
});

test('coordinator workspace selects nodes, reads Radarr directly, and keeps configuration decryption in the browser', () => {
  assert.match(workspaceScript, /Choose a connected fossil or boneyard node/);
  assert.match(workspaceScript, /roleLabel\(role\)/);
  assert.match(workspaceScript, /'coordinator' \? 'roarlay' : role === 'storage' \? 'boneyard' : 'fossil'/);
  assert.match(workspace, />roarlay<\/div>/);
  assert.match(compose, /  roarlay:\n/);
  assert.doesNotMatch(compose, /  coordinator:\n/);
  assert.match(boneyardCompose, /  boneyard:\n/);
  assert.match(tailscaleCompose, /aliases:\n          - coordinator/);
  assert.match(workspaceScript, /ageIdentityInMemory = identity/);
  assert.match(workspaceScript, /arrDirectRequest\('\/radarr\/live-library'\)/);
  assert.doesNotMatch(workspaceScript, /window\.rawrarrMiniExport\.loadRadarrLibrary/);
  assert.match(workspace, /Cluster configuration/);
  assert.doesNotMatch(workspace, /refresh-configuration|Load cluster configuration/);
  assert.match(workspaceScript, /if \(view === 'configuration' && updateHash\) void loadClusterConfiguration\(\)/);
  assert.match(workspaceScript, /if \(location\.hash === '#configuration'\) void loadClusterConfiguration\(\)/);
  assert.match(workspaceScript, /window\.rawrarrMiniExport\.loadSharedConfig\(identity, route\.base\)/);
  assert.match(workspaceScript, /function redactConfigSecrets/);
  assert.match(workspaceScript, /secretConfigKey = .*username/);
  assert.match(workspaceScript, /'<REDACTED>'/);
  assert.match(metadataExport, /loadSharedConfig/);
  assert.match(workspaceScript, /base: `\/nodes\/\$\{encodeURIComponent\(node\.nodeId\)\}`/);
  assert.match(workspaceScript, /movie-count/);
  assert.match(workspaceScript, /poster\.loading = 'lazy'/);
  assert.match(workspace, /id="movies"/);
  assert.match(workspaceScript, /title\.textContent = item\.title \|\| 'Untitled'/);
  assert.match(workspaceScript, /\/api\/workspace\/pending/);
  assert.match(workspaceScript, /\/api\/workspace\/approve/);
  assert.match(coordinator, /fn ciphertext_export_path\(path: &str\)/);
  assert.match(workspaceScript, /sessionStorage\.setItem\(selectedNodeStorageKey, selectedNode\)/);
  assert.match(workspaceScript, /sessionStorage\.getItem\(selectedNodeStorageKey\)/);
  assert.match(workspaceScript, /searchParams\.delete\('node'\)/);
  assert.match(workspaceScript, /if \(selectedNode\) \{[\s\S]*?const current = nodes\.find\(node => node\.nodeId === selectedNode && canOpen\(node\)\)/);
  assert.match(workspaceScript, /if \(!selectedNode && requested\) \{\s*openNode\(requested\);\s*\}/);
  assert.match(workspaceScript, /const directCandidates = nodes[\s\S]*?deviceRouteStatusByNode\.get\(node\.nodeId\)\?\.directRoute/);
  assert.match(workspaceScript, /directCandidates\.sort\(\(a, b\) => a\.route\.latencyMs - b\.route\.latencyMs\)/);
  assert.match(workspaceScript, /fastest\.node\.preferredDirect = fastest\.route/);
  assert.doesNotMatch(workspaceScript, /if \(requested\) openNode\(requested\)/);
  assert.doesNotMatch(workspaceScript, /nodePage/);
  assert.match(coordinator, /if !method_allowed/);
  assert.match(coordinator, /encrypted_session_request\(&path, &parts\.method, &request_body\)/);
});

test('browser and metadata node derive the encrypted library key with matching HKDF parameters', () => {
  assert.match(metadataExport, /generateKey\(\{ name: 'ECDH', namedCurve: 'P-256' \}, true, \['deriveBits'\]\)/);
  assert.match(metadataExport, /deriveBits\(\{ name: 'ECDH', public: nodePublicKey \}, keyPair\.privateKey, 256\)/);
  assert.match(metadataExport, /name: 'HKDF', hash: 'SHA-256', salt: hkdfSalt, info: hkdfInfo/);
  assert.match(metadataExport, /rawrarr-coordinator-session-v1/);
  assert.match(metadataExport, /radarr-library-response/);
  assert.match(mainRs, /Hkdf::<sha2::Sha256>::new\([\s\S]*?Some\(b"rawrarr-coordinator-session-v1"\)/);
  assert.match(mainRs, /hkdf\.expand\(b"radarr-library-response"/);
});

test('coordinator node buttons display accessible direct, relay, and offline route states', () => {
  assert.match(workspaceScript, /checkDeviceRoutes\(nodes\)/);
  assert.match(workspaceScript, /deviceRouteStatusByNode\.get\(node\.nodeId\)/);
  assert.match(deviceRoutes, /status: 'offline'/);
  assert.match(deviceRoutes, /device\.relayAvailable \? 'relay' : 'offline'/);
  assert.match(deviceRoutes, /status, directRoute/);
  assert.match(workspaceScript, /nodeStateLabel = node\.online \? `Online · \$\{routeLabel\}` : routeLabel/);
  assert.match(workspaceScript, /stateLabel\.textContent = latencyLabel \? `Online · \$\{latencyLabel\}` : nodeStateLabel/);
  assert.match(workspaceScript, /if \(!node\.online \|\| !node\.relayAvailable\) return/);
  assert.match(workspaceScript, /if \(!node\.relayAvailable && !node\.directUrl\)/);
  assert.match(workspaceScript, /const canOpenNode = node => node\.relayAvailable \|\| Boolean\(node\.directUrl\)/);
  assert.match(workspace, /\.node-state\.online \.node-state-indicator \{ border-color:#a6e3a1; background:#3bc46b/);
  assert.match(workspace, /\.node-state\.offline \.node-state-indicator \{ border-color:#ff9999; background:#ef5c5c/);
  assert.match(workspace, /\.node-state\.direct \.node-state-indicator \{ border-color:#a6e3a1; background:#3bc46b/);
  assert.match(workspace, /\.node-state\.relay \.node-state-indicator \{ border-color:#78b8ec; background:#528fc0/);
  assert.match(workspace, /\.node-state-indicator \{ flex:0 0 13px; width:13px; height:13px; border:1px solid/);
  assert.match(workspace, /box-shadow:0 0 0 3px #3bc46b40/);
  assert.match(workspace, /box-shadow:0 0 0 3px #ef5c5c40/);
});

test('default metadata selection measures online direct metadata nodes and selects the lowest latency', () => {
  assert.match(workspaceScript, /const metadataNodes = nodes\.filter\(node => node\.role === 'metadata' && \(node\.relayAvailable \|\| node\.directUrl\)\)/);
  assert.match(deviceRoutes, /signal: AbortSignal\.timeout\(2500\)/);
  assert.match(deviceRoutes, /probeCacheMilliseconds = 15_000/);
  assert.match(workspaceScript, /checkDeviceRoutes\(workspaceNodes, \{ refresh \}\)/);
  assert.match(workspaceScript, /stateLabel\.textContent = latencyLabel \? `Online · \$\{latencyLabel\}` : nodeStateLabel/);
  assert.match(workspaceScript, /Direct Tailscale route active \(\$\{latencyLabel\}\)/);
  assert.match(workspaceScript, /const relayRoute = \{ nodeId: node\.nodeId, base: `\/nodes\/\$\{encodeURIComponent\(node\.nodeId\)\}`, direct: false \}/);
});

test('overview distinguishes an available metadata node from no online metadata node', () => {
  assert.match(workspaceScript, /const onlineMetadata = workspaceNodes\.some\(candidate => candidate\.role === 'metadata' && candidate\.online\)/);
  assert.match(workspaceScript, /Fossil node not selected/);
  assert.match(workspaceScript, /No online fossil node/);
});

test('Overview genre charts show the top ten genres and combine the remainder as Other', () => {
  assert.match(workspace, /class="overview-card overview-genre-card"><h2>Radarr genres<\/h2><div id="overview-movie-genres" class="overview-genre-chart/);
  assert.match(workspace, /class="overview-card overview-genre-card"><h2>Sonarr genres<\/h2><div id="overview-series-genres" class="overview-genre-chart/);
  assert.match(workspace, /\.overview-genre-card \{ grid-column:span 2; min-height:370px; padding:24px \}/);
  assert.match(workspaceScript, /const topGenres = genres\.slice\(0, 10\)/);
  assert.match(workspaceScript, /const remaining = genres\.slice\(10\)/);
  assert.match(workspaceScript, /chartGenres\.push\(\['Other', remaining\.reduce\(\(total, \[, count\]\) => total \+ count, 0\)\]\)/);
  assert.match(workspaceScript, /conic-gradient\(\$\{stops\.join\(', '\)\}\)/);
  assert.match(workspaceScript, /element\.replaceChildren\(pie, legend\)/);
});

test('encrypted Radarr browser library preserves genres for Overview charts', async () => {
  assert.match(metadataExport, /genres: Array\.isArray\(item\.genres\) \? item\.genres\.filter\(genre => typeof genre === 'string'\) : \[\]/);
  assert.match(workspaceScript, /currentView === '' \|\| currentView === '#overview' \|\| currentView === 'overview'[\s\S]*?loadArrSettings\('radarr'\)[\s\S]*?loadArrSettings\('sonarr'\)/);
});

test('boneyard selection falls back to an available fossil node for workspace views', () => {
  assert.match(workspaceScript, /selectedNode && selectedNodeDetails\?\.role !== 'metadata' && defaultNode/);
  assert.match(workspaceScript, /Boneyard nodes can be opened explicitly, but they cannot provide the/);
  assert.match(workspaceScript, /coordinator workspace data shown on Overview/);
});
