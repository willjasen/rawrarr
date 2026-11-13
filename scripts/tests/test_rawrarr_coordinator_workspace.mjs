import assert from 'node:assert/strict';
import test from 'node:test';
import { readFile } from 'node:fs/promises';

const root = new URL('../../', import.meta.url);
const read = path => readFile(new URL(path, root), 'utf8');
const [coordinator, auth, nodeAuth, workspace, workspaceScript, deviceRoutes, metadataUi, compose, metadataExport, mainRs, boneyardCompose, coordinatorAvatar, internalDo3Compose] = await Promise.all([
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
  read('docker-compose.storage.yml'),
  read('rust-arr/src/coordinator_avatar.rs'),
  read('deploy/do3/docker-compose.internal-coordinator.yml'),
]);

test('About explains all four identity domains and marks planned capabilities clearly', () => {
  const about = workspace.match(/<section id="about-view"[\s\S]*?<\/section>/)?.[0];
  assert.ok(about, 'About section exists');
  assert.match(about, /Four identity domains/);
  assert.match(about, /<h3>User<\/h3><span class="about-identity-status">Current<\/span>/);
  assert.match(about, /<h3>Device<\/h3><span class="about-identity-status">Current<\/span>/);
  assert.match(about, /<h3>Group<\/h3><span class="about-identity-status planned">Planned<\/span>/);
  assert.match(about, /<h3>App<\/h3><span class="about-identity-status planned">Planned<\/span>/);
  assert.match(about, /config\.aged<\/code> will be encrypted to it/);
  assert.match(about, /Every approved group node can access app IDs for now/);
  assert.match(about, /moving an app between fossil nodes preserves its metadata identity/);
  assert.match(about, /Tailscale Serve can route node messages, but it carries ciphertext only/);
  assert.doesNotMatch(about, /AGE-SECRET-KEY|PRIVATE KEY|AGE-SECRET-KEY-PQ/);
});

test('rawrs Talk stays available from a persistent bottom-left voice panel', () => {
  assert.match(workspace, /id="rawrs-launcher" class="rawrs-launcher"[^>]*aria-controls="ai-voice-panel"/);
  assert.match(workspace, /id="rawrs-launcher"[^>]*aria-label="Open rawrs voice agent"[^>]*>rawr<\/button>/);
  assert.match(workspace, /id="ai-voice-panel" class="rawrs-voice-panel"[^>]*hidden/);
  assert.match(workspace, /\.rawrs-launcher \{ position:fixed;[^}]*bottom:20px/);
  assert.match(workspace, /@media\(max-width:640px\) \{ \.rawrs-launcher \{ left:12px/);
  assert.match(workspace, /id="ai-talk" type="button"/);
  assert.match(workspace, /id="ai-voice-transcript"/);
  assert.match(workspace, /id="ai-voice-answer"/);
  assert.doesNotMatch(workspace, /id="ai-tab-voice"/);
});

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

test('left navigation renders every visible option in lowercase', () => {
  const sidebar = workspace.match(/<aside aria-label="Main navigation">([\s\S]*?)<\/aside>/)?.[1];
  assert.ok(sidebar, 'main navigation sidebar exists');
  assert.match(workspace, /\.nav-text\s*\{[^}]*text-transform:\s*lowercase/i);
  const labels = [...sidebar.matchAll(/<span class="nav-text">([^<]+)<\/span>/g)].map(([, label]) => label);
  assert.deepEqual(labels.map(label => label.toLowerCase()), ['overview', 'subscription', 'roarlay', 'fossils', 'discover', 'movies', 'tv', 'music', 'rawrs', 'configuration', 'settings', 'about']);
  assert.match(sidebar, /<a class="nav-item" href="\/billing" aria-label="Subscription"[\s\S]*?<span class="nav-text">Subscription<\/span><\/a>/);
});

test('Roarlay has a dedicated coordinator page and uses the same-origin health check', () => {
  assert.match(workspace, /data-view="roarlay"[^>]*>[\s\S]*?<span class="nav-text">Roarlay<\/span>/);
  assert.match(workspace, /<section id="roarlay-view" hidden>[\s\S]*?Coordinator node[\s\S]*?id="roarlay-health-label"[\s\S]*?id="roarlay-health-refresh"/);
  assert.match(workspaceScript, /'overview', 'roarlay', 'fossils'/);
  assert.match(workspaceScript, /if \(view === 'roarlay'\) void checkRoarlayHealth\(\)/);
  assert.match(workspaceScript, /fetch\('\/health', \{ cache: 'no-store'/);
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

test('metadata device cards display verified Merkle freshness from the node peer status route', () => {
  assert.match(workspaceScript, /\/nodes\/\$\{encodeURIComponent\(node\.nodeId\)\}\/api\/config-peer\/status/);
  assert.match(workspaceScript, /state: 'unreachable'/);
  assert.match(workspaceScript, /Merkle up to date/);
  assert.match(workspaceScript, /Merkle sync incomplete/);
  assert.match(workspaceScript, /Merkle peers unreachable/);
  assert.match(workspaceScript, /merkleStatus\.matchingPeers/);
  assert.match(mainRs, /\.route\("\/api\/config-peer\/status", get\(config_peer_status_handler\)\)/);
  assert.match(mainRs, /async fn config_peer_status_handler\(State\(state\): State<Arc<AppState>>\)/);
  assert.match(coordinator, /path == "api\/config-peer\/status"/);
});

test('Fossils shows a network diagram with one link for each node pair', () => {
  assert.match(workspace, /<section id="fossils-view"[^>]*><div class="devices-page-heading"><h1 class="page-title">Fossils<\/h1><a class="button-link" href="\/setup\/node">Add a fossil<\/a><\/div>/);
  assert.match(workspace, /<section id="device-topology" class="topology-section" aria-labelledby="device-topology-title" hidden>[\s\S]*?<h2 id="device-topology-title">Topology<\/h2>/);
  assert.match(workspace, /<div id="device-topology-content"><\/div>/);
  assert.match(workspace, /id="signing-topology-title">Signing key topology<\/h3>/);
  assert.match(workspace, /You are the authority for this instance/);
  assert.match(workspaceScript, /function renderSigningKeyTopology\(nodes\)/);
  assert.match(workspaceScript, /You · instance signing root/);
  assert.match(workspaceScript, /legacy · no root-signed key/);
  assert.match(coordinator, /signing_root_fingerprint: Option<String>/);
  assert.match(workspaceScript, /function renderDeviceTopology\(nodes, observedPeers, deviceNames, peerStatuses\)/);
  assert.match(workspaceScript, /deviceNames\[node\.nodeId\] \|\| \(node\.displayName && node\.displayName !== node\.nodeId \? node\.displayName : roleLabel\(node\.role\)\)/);
  assert.match(workspaceScript, /deviceNameTokens\(node\.displayName\)/);
  assert.match(workspaceScript, /getElementById\('device-topology-content'\)/);
  assert.match(workspaceScript, /nodes\.length \* \(nodes\.length - 1\) \/ 2/);
  assert.match(workspaceScript, /for \(let leftIndex = 0; leftIndex < count; leftIndex \+= 1\)/);
  assert.match(workspaceScript, /for \(let rightIndex = leftIndex \+ 1; rightIndex < count; rightIndex \+= 1\)/);
  assert.match(workspaceScript, /const tailnetReachable = leftCanReachRight \|\| rightCanReachLeft/);
  assert.match(workspaceScript, /leftStatus\?\.state !== 'stale'/);
  assert.match(workspaceScript, /rightStatus\?\.state !== 'stale'/);
  assert.match(workspaceScript, /tailnetReachable \? ' connected'/);
  assert.match(workspaceScript, /Reachable over tailnet/);
  assert.match(workspaceScript, /Add another device to see its links in the network diagram\./);
  assert.match(workspace, /\.topology-edge\.connected \{ stroke:var\(--green\); stroke-width:3 \}/);
  assert.match(workspace, /\.topology-edge\.disconnected \{ stroke:#ef7777 \}/);
});

test('Fossils layout wraps cards, statuses, actions, and topology at mobile widths', () => {
  assert.match(workspace, /\.devices-page-heading \{ align-items:flex-start; gap:10px \}/);
  assert.match(workspace, /\.device-card \{ box-sizing:border-box; min-width:0; flex-basis:100%; width:100%; max-width:none/);
  assert.match(workspace, /\.device-heading \{ flex-wrap:wrap/);
  assert.match(workspace, /\.node-state \{ flex:0 1 auto; max-width:100%; white-space:normal; overflow-wrap:anywhere/);
  assert.match(workspace, /\.device-edit-button \{ min-height:40px/);
  assert.match(workspace, /\.topology-device \{ width:min\(140px,30%\)/);
  assert.match(workspace, /\.topology-device strong,\.topology-device small \{ min-width:0; overflow-wrap:anywhere \}/);
});

test('Fossils uses the new page hash and redirects the old Devices bookmark', () => {
  assert.match(workspaceScript, /name === 'devices' \? 'fossils' : name/);
  assert.match(workspaceScript, /location\.hash === '#devices'[\s\S]*#fossils/);
  assert.match(workspaceScript, /\['overview', 'roarlay', 'fossils', 'movies'/);
});

test('Devices keeps topology hidden until an enrolled device is online', () => {
  assert.match(workspaceScript, /const topologySection = document\.getElementById\('device-topology'\);\s*emptyDevices\.hidden = nodes\.length !== 0;\s*topologySection\.hidden = !nodes\.some\(node => node\.online === true\);/);
  assert.match(workspaceScript, /if \(nodes\.some\(node => node\.online === true\)\) \{\s*renderDeviceTopology\(nodes, observedSyncthingPeers, deviceNames, configPeerStatusByNode\);\s*\}/);
  assert.match(workspace, /<section id="device-topology"[^>]*hidden/);
});

test('device names open the existing edit dialog from the device card', () => {
  assert.match(workspaceScript, /editButton\.textContent = 'Edit'/);
  assert.match(workspaceScript, /editButton\.addEventListener\('click', event => \{[\s\S]*?deviceEditDialog\.showModal\(\)/);
  assert.match(workspaceScript, /card\.append\(editButton\)/);
  assert.match(workspace, /<dialog id="device-edit-dialog"[\s\S]*?id="device-edit-name"/);
  assert.match(workspaceScript, /saveDeviceName\(ageIdentity/);
  assert.match(workspace, /\.device-edit-dialog \{ width:min/);
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
  assert.match(workspaceScript, /body: JSON\.stringify\(\{ identityEnvelope, rememberOnDevice: false \}\)/);
  assert.match(workspaceScript, /apiKey: document\.getElementById\(copy\.key\)\.value/);
  assert.match(workspaceScript, /uiUsername: document\.getElementById\(copy\.uiUsername\)\.value/);
  assert.match(mainRs, /\.route\("\/api\/settings\/age", get\(get_age_settings\)\.post\(unlock_age_settings\)\)/);
  assert.match(mainRs, /\.route\(\s*"\/api\/settings\/sonarr",[\s\S]*?\.delete\(sonarr::clear\)/);
  assert.match(mainRs, /let direct_protected = Router::new\(\)[\s\S]*?\.route\("\/api\/sonarr\/status", get\(sonarr::status\)\)[\s\S]*?\.route\("\/api\/sonarr\/series", get\(sonarr::series\)\)/);
  assert.match(mainRs, /\.route\(\s*"\/api\/settings\/radarr",[\s\S]*?\.delete\(delete_radarr_settings\)/);
  assert.doesNotMatch(coordinator, /"api\/settings\/sonarr"/);
});

test('encrypted metadata fetches include the direct node browser session cookie', () => {
  assert.match(metadataExport, /async function fetchBytes\(url, \{ signal \} = \{\}\)[\s\S]*?signal,[\s\S]*?credentials: 'include'/);
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
  assert.match(workspace, /After validation, Rawrarr saves your ID in this browser when browser storage is available/);
  assert.match(workspace, /id="age-storage-title">Age private ID in this browser/);
  assert.match(workspace, /id="age-storage-status"/);
  assert.match(workspace, /id="save-age-id"/);
  assert.match(workspace, /id="remove-age-id"/);
  assert.match(workspace, /id="show-age-recipient" type="button" hidden>Show age public ID<\/button>/);
  assert.match(workspace, /id="age-recipient-display" class="age-recipient-display" hidden/);
  assert.match(workspace, /id="copy-age-recipient" type="button">Copy age public ID/);
  assert.match(workspace, /id="clear-key" type="button" hidden/);
  assert.match(workspace, /<title>rawrarr<\/title>/);
  assert.match(workspace, /saved in this browser/);
  assert.match(workspaceScript, /let ageIdentityInMemory = rememberedAgeIdentity/);
  assert.match(workspaceScript, /ageIdentityInMemory = identity/);
  assert.match(workspaceScript, /addEventListener\('pagehide', \(\) =>/);
  assert.match(workspaceScript, /localStorage\.getItem\(ageSessionKey\)/);
  assert.match(workspaceScript, /localStorage\.setItem\(ageSessionKey, identity\)/);
  assert.match(workspaceScript, /deriveProfileRecipient\(identity\)[\s\S]*?rememberAgeIdentity\(identity\)[\s\S]*?await unlockMetadataVaultForSettings\(\)/);
  assert.match(workspaceScript, /try \{\s*localStorage\.setItem\(ageSessionKey, identity\);[\s\S]*?return false;/);
  assert.match(workspaceScript, /this browser could not save it/);
  assert.match(workspaceScript, /localStorage\.removeItem\(ageSessionKey\)/);
  assert.match(workspaceScript, /showAgeRecipientButton\.addEventListener\('click', async \(\) =>/);
  assert.match(workspaceScript, /deriveProfileRecipient\(identity\)[\s\S]*?agePublicRecipient = recipient;[\s\S]*?ageRecipientShort\.textContent = recipient\.length/);
  assert.match(workspaceScript, /copyAgeRecipientButton\.addEventListener\('click', async \(\) =>/);
  assert.match(workspaceScript, /navigator\.clipboard\.writeText\(recipient\)/);
  assert.match(workspaceScript, /Age public ID copied\./);
  assert.match(workspaceScript, /agePublicRecipient = '';\s*ageRecipientShort\.textContent = '';/);
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
  assert.match(workspaceScript, /const sign = decision === 'reject' \? signDeviceRejection : signDeviceApproval/);
  assert.match(workspaceScript, /signedPayload: approval\.signedPayload/);
  assert.match(workspaceScript, /approveNodeDialog\.addEventListener\('close'/);
  assert.match(workspaceScript, /approveNodeDialog\.close\(\)/);
  assert.match(workspace, /id="approve-node-attestation"/);
  assert.match(workspaceScript, /You are signing off on this device/);
  assert.match(workspaceScript, /Your root signature authorizes the exact device key and role below/);
  assert.match(workspaceScript, /signDeviceRejection/);
  assert.match(signingClient, /rawrarr-device-rejection-v1/);
  assert.match(workspaceScript, /permanently rejecting this device identity/);
  assert.match(coordinator, /rawrarr_rejected_nodes WHERE node_id=\$1 OR signing_fingerprint=\$2/);
  assert.match(coordinator, /rawrarr-device-rejection-v1/);
  assert.match(coordinator, /INSERT INTO rawrarr_rejected_nodes/);
  assert.match(signingClient, /secretKey\?\.fill\(0\)/);
  assert.match(onboarding, /id="root-fingerprint-confirm"/);
  assert.match(onboarding, /Re-enter the root fingerprint you saved separately/);
  assert.match(onboardingScript, /rootFingerprint: pinnedFingerprint/);
  assert.match(subscriberComposeSource, /RAWARR_INSTANCE_ROOT_FINGERPRINT/);
  assert.match(onboardingScript, /pagehide/);
  assert.match(onboardingScript, /localStorage\.getItem\('rawrarr\.browser\.age-id'\)/);
  assert.doesNotMatch(onboardingScript, /localStorage\.(?:setItem|removeItem)|sessionStorage/);
});

test('signed-in profile requests a coordinator-rendered JPEG XL dinosaur from the public age ID', () => {
  assert.match(coordinator, /\.route\("\/api\/workspace\/profile", get\(workspace_profile\)\)/);
  assert.match(coordinator, /SELECT id, to_char\(trial_started_at AT TIME ZONE 'UTC'[\s\S]*?trial_started_at \+ interval '3 days'[\s\S]*?WHERE id=\$1/);
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
  assert.match(workspaceScript, /const identity = ageIdentityInMemory;/);
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
  assert.match(coordinator, /avatar_cache: Arc<RwLock<HashMap<String, Arc<Vec<u8>>>>/);
  assert.match(coordinator, /async fn cached_workspace_avatar/);
  assert.match(coordinator, /profile:jxl:v1:\{seed\}/);
  assert.match(coordinator, /profile:png:v1:\{seed\}/);
  assert.match(coordinator, /device:jxl:v1:\{seed\}/);
  assert.match(coordinator, /device:png:v1:\{seed\}/);
  assert.match(coordinator, /cache\.len\(\) >= 256/);
  assert.match(workspaceScript, /Generated dinosaur profile picture/);
  assert.doesNotMatch(workspaceScript, /Generated in this browser from your public age ID\. No image is uploaded\./);
  assert.match(workspaceScript, /profile-avatar-status'\)\.textContent = ''/);
  assert.doesNotMatch(coordinator, /SELECT\s+(?:profile_picture|avatar_seed)/i);
});

test('profile retries a temporary 404 and preserves a loaded username through later profile errors', () => {
  assert.match(workspaceScript, /for \(let attempt = 0; attempt < 3; attempt \+= 1\)[\s\S]*?error\.status !== 404 \|\| attempt === 2[\s\S]*?setTimeout\(resolve, 250 \* \(attempt \+ 1\)\)/);
  assert.match(workspaceScript, /error\.status = response\.status/);
  assert.match(workspaceScript, /if \(!profileBaseName\) \{\s*document\.getElementById\('profile-name'\)\.textContent = error\.message/);
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

test('internal coordinator maps current fossil IDs to their relay and direct Tailscale routes', () => {
  assert.match(internalDo3Compose, /"device-398f0ce833b6d7e2b9e8f760":"https:\/\/rawrarr-macbook-pro\.risk-mermaid\.ts\.net\/"/);
  assert.match(internalDo3Compose, /"device-398f0ce833b6d7e2b9e8f760":"https:\/\/rawrarr-macbook-pro\.risk-mermaid\.ts\.net\/direct-api\/"/);
  assert.match(internalDo3Compose, /"device-d3ee53b1d561fb349955b199":"https:\/\/rawrarr-macbook-pro\.risk-mermaid\.ts\.net\/"/);
  assert.match(internalDo3Compose, /"device-d3ee53b1d561fb349955b199":"https:\/\/rawrarr-macbook-pro\.risk-mermaid\.ts\.net\/direct-api\/"/);
  assert.match(internalDo3Compose, /"device-8de78fadda6400a45ca1a30b":"https:\/\/rawrarr-mac-mini\.risk-mermaid\.ts\.net\/"/);
  assert.match(internalDo3Compose, /"device-8de78fadda6400a45ca1a30b":"https:\/\/rawrarr-mac-mini\.risk-mermaid\.ts\.net\/direct-api\/"/);
  assert.match(internalDo3Compose, /"device-9288eb3b039608d16a9102a5"/);
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
  assert.doesNotMatch(workspace, /<header[^>]*>\s*<div class="brand"[^>]*>roarlay<\/div>/);
  assert.match(workspace, /<button id="sign-out" type="button">Sign out<\/button>/);
  assert.match(compose, /  roarlay:\n/);
  assert.doesNotMatch(compose, /  coordinator:\n/);
  assert.match(boneyardCompose, /  boneyard:\n/);
  assert.match(internalDo3Compose, /network_mode: service:tailscale/);
  assert.match(workspaceScript, /ageIdentityInMemory = identity/);
  assert.match(workspaceScript, /arrDirectRequest\('\/radarr\/live-library'\)/);
  assert.doesNotMatch(workspaceScript, /window\.rawrarrMiniExport\.loadRadarrLibrary/);
  assert.match(workspace, /Cluster configuration/);
  assert.doesNotMatch(workspace, /refresh-configuration|Load cluster configuration/);
  assert.match(workspaceScript, /if \(view === 'configuration' && updateHash\) void loadClusterConfiguration\(\)/);
  assert.match(workspaceScript, /if \(location\.hash === '#configuration'\) void loadClusterConfiguration\(\)/);
  assert.match(workspaceScript, /window\.rawrarrMiniExport\.loadSharedConfig\(identity, route\.base\)/);
  assert.match(workspaceScript, /if \(!route \|\| typeof route\.base !== 'string' \|\| !route\.base\.trim\(\)\)/);
  assert.match(workspaceScript, /function redactConfigSecrets/);
  assert.match(workspaceScript, /secretConfigKey = .*username/);
  assert.match(workspaceScript, /'<REDACTED>'/);
  assert.match(metadataExport, /loadSharedConfig/);
  assert.match(metadataExport, /typeof base !== 'string' \|\| !base\.trim\(\)/);
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
  assert.match(workspaceScript, /stateLabel\.textContent = latencyLabel \|\| nodeStateLabel/);
  assert.doesNotMatch(workspaceScript, /async function loadNodeAvatar\(node, pictureSource, image\) \{\s*if \(!node\.online \|\| !node\.relayAvailable\)/);
  assert.match(workspaceScript, /if \(!node\.relayAvailable && !node\.directUrl\)/);
  assert.match(workspaceScript, /const canOpenNode = node => node\.relayAvailable \|\| Boolean\(node\.directUrl\)/);
  assert.match(workspace, /\.node-state\.online \.node-state-indicator \{ border-color:#a6e3a1; background:#3bc46b/);
  assert.match(workspace, /\.node-state\.offline \.node-state-indicator \{ border-color:#ff9999; background:#ef5c5c/);
  assert.match(workspace, /\.node-state\.online\.offline \.node-state-indicator \{ border-color:#d5b66a; background:#b98c27/);
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
  assert.match(workspaceScript, /stateLabel\.textContent = latencyLabel \|\| nodeStateLabel/);
  assert.match(workspaceScript, /Direct Tailscale route active \(\$\{latencyLabel\}\)/);
  assert.match(workspaceScript, /const relayRoute = \{ nodeId: node\.nodeId, base: `\/nodes\/\$\{encodeURIComponent\(node\.nodeId\)\}`, direct: false \}/);
});

test('device cards hide unavailable Syncthing health until configured', () => {
  assert.doesNotMatch(workspaceScript, /Syncthing health unavailable/);
  assert.match(workspaceScript, /else if \(!report\?\.configured\) \{\s*syncHealth\.hidden = true;/);
});

test('overview distinguishes an available metadata node from no online metadata node', () => {
  assert.match(workspaceScript, /const onlineMetadata = workspaceNodes\.some\(candidate => candidate\.role === 'metadata' && candidate\.online\)/);
  assert.match(workspaceScript, /Fossil node not selected/);
  assert.match(workspaceScript, /No online fossil node/);
});

test('Overview genre charts show the top ten genres and combine the remainder as Other', () => {
  assert.match(workspace, /id="overview-radarr-genres-card" class="overview-card overview-genre-card" hidden><h2>Radarr genres<\/h2><div id="overview-movie-genres" class="overview-genre-chart/);
  assert.match(workspace, /id="overview-sonarr-genres-card" class="overview-card overview-genre-card" hidden><h2>Sonarr genres<\/h2><div id="overview-series-genres" class="overview-genre-chart/);
  assert.match(workspace, /id="overview-radarr-card" class="overview-card" hidden><h2>Radarr<\/h2>/);
  assert.match(workspace, /id="overview-sonarr-card" class="overview-card" hidden><h2>Sonarr<\/h2>/);
  assert.match(workspaceScript, /overviewCards: \['overview-radarr-card', 'overview-radarr-genres-card'\]/);
  assert.match(workspaceScript, /overviewCards: \['overview-sonarr-card', 'overview-sonarr-genres-card'\]/);
  assert.match(workspaceScript, /for \(const id of fields\.overviewCards\) document\.getElementById\(id\)\.hidden = true;[\s\S]*?for \(const id of fields\.overviewCards\) document\.getElementById\(id\)\.hidden = !settings\.configured/);
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
