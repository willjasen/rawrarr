import { checkDeviceRoutes } from './coordinator-device-routes.mjs';
import { deriveNodeAvatarSeed } from './coordinator-node-avatar.mjs';
import { dinosaurMainColor, dinosaurWingWord } from './profile-color.mjs';
import { initAiChat } from './coordinator-ai.mjs';

const status = document.getElementById('status');
const nodesPanel = document.getElementById('nodes');
const pendingPanel = document.getElementById('pending');
const contentPanel = document.getElementById('content');
const contentStatus = document.getElementById('content-status');
const moviesPanel = document.getElementById('movies');
let aiMovies = null;
let aiSeries = null;
const libraryTextCachePrefix = 'rawrarr.library-text.v1';
function libraryTextCacheKey(nodeId, mediaType) {
  return `${libraryTextCachePrefix}:${encodeURIComponent(nodeId)}:${mediaType}`;
}
function readLibraryTextCache(nodeId, mediaType) {
  if (!nodeId) return null;
  try {
    const cached = JSON.parse(localStorage.getItem(libraryTextCacheKey(nodeId, mediaType)) || 'null');
    if (!cached || !Array.isArray(cached.items)) return null;
    return cached.items;
  } catch { return null; }
}
function writeLibraryTextCache(nodeId, mediaType, items) {
  if (!nodeId || !Array.isArray(items)) return;
  try {
    localStorage.setItem(libraryTextCacheKey(nodeId, mediaType), JSON.stringify({ savedAt: Date.now(), items }));
  } catch { /* Storage can be disabled or full; live library rendering still works. */ }
}
const aiChat = initAiChat({
  getMovies: () => aiMovies,
  getSeries: () => aiSeries,
  loadSeries: () => loadSonarrSeries(),
  openMovies: () => showView('movies'),
});
const ARR_SUMMARY_DEFINITIONS = {
  radarr: {
    containerId: 'radarr-summary',
    cards: [
      { key: 'libraryCount', label: 'Movies in Radarr', detail: 'Tracked library entries', valueId: 'movie-count' },
      { key: 'filesPresent', label: 'Files present', detail: 'Movies with an imported file', valueId: 'movie-files' },
      { key: 'diskUsage', label: 'Disk usage', detail: 'Size reported by Radarr', valueId: 'movie-size' },
      { key: 'metadataSize', label: 'Metadata size', detail: 'Radarr movie metadata', valueId: 'movie-metadata-size', secondary: { key: 'allMetadataSize', label: 'All metadata', valueId: 'all-metadata-size' } },
    ],
  },
  sonarr: {
    containerId: 'sonarr-summary',
    cards: [
      { key: 'libraryCount', label: 'Series in Sonarr', detail: 'Tracked library entries', valueId: 'tv-series-count' },
      { key: 'filesPresent', label: 'Episodes present', detail: 'Imported episodes', valueId: 'tv-episodes-present' },
      { key: 'diskUsage', label: 'Disk usage', detail: 'Size reported by Sonarr', valueId: 'tv-series-size' },
      { key: 'metadataSize', label: 'Metadata size', detail: 'Sonarr series metadata', valueId: 'tv-metadata-size', secondary: { key: 'allMetadataSize', label: 'All metadata', valueId: 'tv-all-metadata-size' } },
    ],
  },
};

function renderArrSummaryCards(service) {
  const definition = ARR_SUMMARY_DEFINITIONS[service];
  const container = document.getElementById(definition.containerId);
  container.replaceChildren();
  for (const card of definition.cards) {
    const article = document.createElement('article'); article.className = 'metric';
    const label = document.createElement('div'); label.className = 'metric-label'; label.textContent = card.label;
    const value = document.createElement('div'); value.className = 'metric-value'; value.id = card.valueId; value.textContent = '—';
    const detail = document.createElement('div'); detail.className = 'sub'; detail.textContent = card.detail;
    article.append(label, value, detail);
    if (card.secondary) {
      const secondary = document.createElement('div'); secondary.className = 'sub';
      secondary.append(`${card.secondary.label}: `);
      const secondaryValue = document.createElement('span'); secondaryValue.id = card.secondary.valueId; secondaryValue.textContent = '—';
      secondary.append(secondaryValue);
      article.append(secondary);
    }
    container.append(article);
  }
}

function setArrSummaryValues(service, values) {
  const cards = ARR_SUMMARY_DEFINITIONS[service].cards;
  for (const [key, value] of Object.entries(values)) {
    const card = cards.find(candidate => candidate.key === key || candidate.secondary?.key === key);
    const valueId = card?.key === key ? card.valueId : card?.secondary?.valueId;
    if (valueId) document.getElementById(valueId).textContent = value;
  }
}

function resetArrSummary(service) {
  const values = {};
  for (const card of ARR_SUMMARY_DEFINITIONS[service].cards) {
    values[card.key] = '—';
    if (card.secondary) values[card.secondary.key] = '—';
  }
  setArrSummaryValues(service, values);
}

renderArrSummaryCards('radarr');
renderArrSummaryCards('sonarr');
const ageInput = document.getElementById('age-id');
const ageIdEntry = document.getElementById('age-id-entry');
const rememberAgeId = document.getElementById('remember-age-id');
const clearKeyButton = document.getElementById('clear-key');
const ageStorageStatus = document.getElementById('age-storage-status');
const ageStorageMessage = document.getElementById('age-storage-message');
const settingsAgeIdInput = document.getElementById('settings-age-id');
const saveAgeIdButton = document.getElementById('save-age-id');
const removeAgeIdButton = document.getElementById('remove-age-id');
const views = Object.fromEntries(['overview', 'devices', 'movies', 'series', 'assistant', 'configuration', 'settings', 'about'].map(name => [name, document.getElementById(`${name}-view`)]));
function roleLabel(role) {
  return role === 'coordinator' ? 'roarlay' : role === 'storage' ? 'boneyard' : 'fossil';
}
function showView(name, updateHash = true) {
  const requestedView = name === 'tv' ? 'series' : name;
  const view = views[requestedView] ? requestedView : 'overview';
  for (const [key, element] of Object.entries(views)) element.hidden = key !== view;
  for (const button of document.querySelectorAll('.nav-item[data-view]')) {
    const active = button.dataset.view === view;
    button.classList.toggle('active', active);
    if (active) button.setAttribute('aria-current', 'page'); else button.removeAttribute('aria-current');
  }
  if (updateHash) history.replaceState(null, '', `${location.pathname}${location.search}#${view === 'series' ? 'tv' : view}`);
  if (view === 'configuration' && updateHash) void loadClusterConfiguration();
  if (view === 'movies' && updateHash) {
    void loadArrSettings('radarr');
    // Settings can report an ID persisted by this browser even when this tab's
    // in-memory copy was cleared by pagehide. Rehydrate it before unlocking.
    if (!ageIdentityInMemory) ageIdentityInMemory = localStorage.getItem(ageSessionKey) || '';
    updateAgeIdControls();
    if (selectedNode) renderCachedMovies(selectedNode);
    if (ageIdentityInMemory && selectedNode) void unlockMetadata();
  }
  if (view === 'series' && updateHash) {
    const cachedSeries = ageIdentityInMemory ? readLibraryTextCache(selectedNode, 'tv') : null;
    if (cachedSeries) {
      renderSonarrSeries(cachedSeries, 'Cached TV library. Updating…');
      void attachSonarrPosters(document.getElementById('sonarr-series'));
    }
    void loadArrSettings('sonarr');
    if (selectedNodeDetails) void chooseNodeRoute(selectedNodeDetails, true);
  }
  if (view === 'settings' && updateHash) {
    void loadArrSettings('radarr');
    void loadArrSettings('sonarr');
  }
  if (view === 'overview' && updateHash) {
    void loadOverviewMetadata();
    if (selectedNodeDetails?.role === 'metadata') {
      void loadArrSettings('radarr');
      void loadArrSettings('sonarr');
    }
  }
  if (view === 'assistant') void aiChat.refresh();
  if ((view === 'movies' || view === 'series') && updateHash) void loadMetadataSizes();
}
document.querySelectorAll('.nav-item[data-view]').forEach(button => button.addEventListener('click', () => showView(button.dataset.view)));
document.getElementById('configure-sonarr')?.addEventListener('click', () => showView('settings'));
showView(location.hash.slice(1) || 'overview', false);
if (location.hash === '#series') history.replaceState(null, '', `${location.pathname}${location.search}#tv`);
let profileAvatarSeed = '';
let profileAvatarLoadNode = '';
let profileBaseName = '';
function renderProfileAvatar() {
  if (!profileAvatarSeed) return;
  const [baseFirst = '', baseSecond = ''] = profileBaseName.split('-');
  const words = [dinosaurMainColor(profileAvatarSeed), dinosaurWingWord(profileAvatarSeed), baseFirst, baseSecond];
  const profileName = document.getElementById('profile-name');
  profileName.replaceChildren(...[
    document.createTextNode(words.slice(0, 2).join(' ')),
    document.createElement('br'),
    document.createTextNode(words.slice(2).join(' ')),
  ]);
  const sidebarName = document.getElementById('sidebar-profile-name');
  sidebarName.replaceChildren(...words.map(word => {
    const part = document.createElement('span');
    part.textContent = word;
    return part;
  }));
  sidebarName.setAttribute('aria-label', words.join(' '));
  const picture = document.getElementById('profile-picture');
  const jxlUrl = `/api/workspace/profile-picture/${profileAvatarSeed}/jxl`;
  const pngUrl = `/api/workspace/profile-picture/${profileAvatarSeed}/png`;
  document.getElementById('profile-picture-jxl').srcset = jxlUrl;
  picture.src = pngUrl;
  picture.alt = `Generated dinosaur profile picture for ${words.join(' ')}`;
  document.getElementById('sidebar-profile-picture-jxl').srcset = jxlUrl;
  const sidebarPicture = document.getElementById('sidebar-profile-picture');
  sidebarPicture.src = pngUrl;
  sidebarPicture.alt = '';
  document.getElementById('profile-avatar-status').textContent = '';
}
async function loadProfileAvatar() {
  if (profileAvatarSeed) { renderProfileAvatar(); return; }
  const node = workspaceNodes.find(candidate => candidate.role === 'metadata' && candidate.online && candidate.relayAvailable)
    || (selectedNodeDetails?.role === 'metadata' ? selectedNodeDetails : null);
  if (!node || profileAvatarLoadNode === node.nodeId) return;
  profileAvatarLoadNode = node.nodeId;
  try {
    const response = await fetch(`/nodes/${encodeURIComponent(node.nodeId)}/api/metadata/export/recipient`, { cache: 'no-store', headers: { Accept: 'text/plain' } });
    if (!response.ok) throw new Error('Public age ID is not available from this fossil node.');
    const recipient = (await response.text()).trim();
    if (!/^age1pq1[a-z0-9]+$/.test(recipient)) throw new Error('Fossil node returned an invalid public age ID.');
    const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(recipient)));
    profileAvatarSeed = Array.from(digest, byte => byte.toString(16).padStart(2, '0')).join('');
    renderProfileAvatar();
  } catch (error) {
    document.getElementById('profile-avatar-status').textContent = error.message || 'Could not load the public age ID.';
    profileAvatarLoadNode = '';
    const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(`rawrarr-profile:${profileBaseName}`)));
    profileAvatarSeed = Array.from(digest, byte => byte.toString(16).padStart(2, '0')).join('');
    renderProfileAvatar();
  }
}
async function loadNodeAvatar(node, pictureSource, image) {
  if (!node.online || !node.relayAvailable) return;
  let seedPromise = nodeAvatarSeeds.get(node.nodeId);
  if (!seedPromise) {
    seedPromise = deriveNodeAvatarSeed(node.nodeId).catch(() => '');
    nodeAvatarSeeds.set(node.nodeId, seedPromise);
  }
  const seed = await seedPromise;
  if (!seed) return;
  pictureSource.srcset = `/api/workspace/profile-picture/${seed}/jxl`;
  image.src = `/api/workspace/profile-picture/${seed}/png`;
}
async function loadProfile() {
  try {
    const profile = await jsonRequest('/api/workspace/profile');
    profileBaseName = profile.displayName || 'Rawrarr user';
    document.getElementById('profile-name').textContent = profileBaseName;
    document.getElementById('sidebar-profile-name').textContent = profileBaseName;
    await loadProfileAvatar();
  } catch (error) {
    document.getElementById('profile-name').textContent = error.message;
    document.getElementById('sidebar-profile-name').textContent = 'Profile unavailable';
  }
}
void loadProfile();
let selectedNode = null;
let selectedNodeDetails = null;
let selectedRoute = null;
let selectedRoutePromise = null;
let selectedRoutePromiseNode = null;
let deviceNames = {};
let savingDeviceName = false;
let workspaceNodes = [];
const storageScanStatusByNode = new Map();
const storageScansActive = new Set();
const syncthingHealthCache = new Map();
const nodeAvatarSeeds = new Map();
let deviceRouteStatusByNode = new Map();
function deviceNameTokens(value) {
  return new Set(String(value || '').toLowerCase().split(/[^a-z0-9]+/).filter(token => token.length >= 4));
}
function renderDeviceTopology(nodes, observedPeers, deviceNames) {
  const panel = document.getElementById('device-topology-content');
  const graph = document.createElement('div');
  graph.className = 'topology-graph';
  graph.setAttribute('role', 'img');
  graph.setAttribute('aria-label', `Network diagram with ${nodes.length} devices and ${nodes.length * (nodes.length - 1) / 2} possible device links`);
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 1000 500');
  svg.setAttribute('preserveAspectRatio', 'none');
  svg.setAttribute('aria-hidden', 'true');
  graph.append(svg);

  const observedEdges = new Map();
  for (const peer of observedPeers) {
    const source = nodes.find(node => node.nodeId === peer.sourceNodeId);
    if (!source) continue;
    const peerTokens = deviceNameTokens(peer.name);
    const targets = nodes.filter(node => node.nodeId !== source.nodeId).filter(node => {
      const tokens = new Set([...deviceNameTokens(node.nodeId), ...deviceNameTokens(deviceNames[node.nodeId]), ...deviceNameTokens(node.displayName)]);
      return [...tokens].some(token => peerTokens.has(token));
    });
    if (targets.length !== 1) continue;
    const target = targets[0];
    const pair = [source.nodeId, target.nodeId].sort();
    const key = `${pair[0]}\u0000${pair[1]}`;
    const existing = observedEdges.get(key);
    if (existing) {
      existing.connected ||= Boolean(peer.connected);
      if (Number.isFinite(peer.completionPercent)) existing.completionPercent = Math.max(existing.completionPercent || 0, peer.completionPercent);
    } else {
      observedEdges.set(key, { connected: Boolean(peer.connected), completionPercent: peer.completionPercent });
    }
  }

  const count = nodes.length;
  const points = nodes.map((_, index) => {
    if (count === 2) return { x: index === 0 ? 320 : 680, y: 250 };
    if (count === 1) return { x: 500, y: 250 };
    const angle = -Math.PI / 2 + (Math.PI * 2 * index / Math.max(count, 1));
    return { x: 500 + Math.cos(angle) * 340, y: 250 + Math.sin(angle) * 175 };
  });
  for (let leftIndex = 0; leftIndex < count; leftIndex += 1) {
    for (let rightIndex = leftIndex + 1; rightIndex < count; rightIndex += 1) {
      const pair = [nodes[leftIndex].nodeId, nodes[rightIndex].nodeId].sort();
      const observed = observedEdges.get(`${pair[0]}\u0000${pair[1]}`);
      const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
      line.setAttribute('x1', String(points[leftIndex].x));
      line.setAttribute('y1', String(points[leftIndex].y));
      line.setAttribute('x2', String(points[rightIndex].x));
      line.setAttribute('y2', String(points[rightIndex].y));
      line.setAttribute('class', `topology-edge${observed ? observed.connected ? ' connected' : ' disconnected' : ''}`);
      svg.append(line);
    }
  }

  nodes.forEach((node, index) => {
    const item = document.createElement('div');
    item.className = `topology-device${node.online ? ' online' : ' offline'}`;
    item.style.left = `${points[index].x / 10}%`;
    item.style.top = `${points[index].y / 5}%`;
    const name = document.createElement('strong');
    name.textContent = deviceNames[node.nodeId] || node.displayName || node.nodeId;
    const role = document.createElement('small');
    role.textContent = `${roleLabel(node.role)} · ${node.online ? 'online' : 'offline'}`;
    item.append(name, role);
    graph.append(item);
  });

  const legend = document.createElement('div');
  legend.className = 'topology-legend';
  legend.innerHTML = '<span><i class="connected-line"></i>Connected</span><span><i class="disconnected-line"></i>Reported disconnected</span><span><i></i>No live peer report</span>';
  if (nodes.length < 2) {
    const empty = document.createElement('p');
    empty.className = 'topology-empty';
    empty.textContent = 'Add another device to see its links in the network diagram.';
    panel.replaceChildren(graph, empty);
    return;
  }
  panel.replaceChildren(graph, legend);
}
const canOpenNode = node => node.relayAvailable || Boolean(node.directUrl);
const selectedNodeStorageKey = 'rawrarr.coordinator.selected-node';
const initialUrl = new URL(window.location.href);
const requestedNode = initialUrl.searchParams.get('node') || sessionStorage.getItem(selectedNodeStorageKey);
if (initialUrl.searchParams.has('node')) {
  initialUrl.searchParams.delete('node');
  window.history.replaceState(null, '', initialUrl);
}
const ageSessionKey = 'rawrarr.browser.age-id';
sessionStorage.removeItem(ageSessionKey);
const metadataExportButton = document.getElementById('export-metadata');
const metadataExportStatus = document.getElementById('metadata-export-status');
const metadataExportDialog = document.getElementById('metadata-export-dialog');
const metadataExportForm = document.getElementById('metadata-export-form');
const metadataExportInput = document.getElementById('metadata-export-identity');
const metadataExportDialogStatus = document.getElementById('metadata-export-dialog-status');
const metadataExportCancel = document.getElementById('metadata-export-cancel');
async function runMetadataExport(identity) {
  const node = selectedNodeDetails;
  if (!node || node.role !== 'metadata') {
    metadataExportStatus.textContent = 'Select a fossil node before exporting.';
    return;
  }
  metadataExportButton.disabled = true;
  try {
    await import('/assets/metadata-export.js');
    const route = selectedRoute?.nodeId === node.nodeId ? selectedRoute : await chooseNodeRoute(node, true);
    const count = await window.rawrarrMiniExport.runMiniExport(identity, message => { metadataExportStatus.textContent = message; }, route.base);
    metadataExportStatus.textContent = `Downloaded ${count} movie metadata record(s) and matching available artwork.`;
  } catch (error) {
    metadataExportStatus.textContent = error instanceof Error ? error.message : 'Metadata export failed.';
  } finally {
    metadataExportButton.disabled = false;
  }
}
metadataExportButton?.addEventListener('click', () => {
  metadataExportStatus.textContent = '';
  if (ageIdentityInMemory) {
    void runMetadataExport(ageIdentityInMemory);
    return;
  }
  metadataExportInput.value = '';
  metadataExportDialogStatus.textContent = '';
  metadataExportDialog.showModal();
  metadataExportInput.focus();
});
metadataExportCancel?.addEventListener('click', () => metadataExportDialog.close());
metadataExportDialog?.addEventListener('close', () => {
  metadataExportInput.value = '';
  metadataExportDialogStatus.textContent = '';
});
metadataExportForm?.addEventListener('submit', async event => {
  event.preventDefault();
  const identity = metadataExportInput.value.trim();
  if (!identity) return;
  ageIdentityInMemory = identity;
  updateAgeIdControls();
  metadataExportInput.value = '';
  metadataExportDialog.close();
  await runMetadataExport(identity);
});

// Remembering is opt-in; otherwise the age ID exists only in this tab's memory.
let ageIdentityInMemory = localStorage.getItem(ageSessionKey) || '';
rememberAgeId.checked = Boolean(ageIdentityInMemory);
ageInput.value = '';

function updateAgeIdControls() {
  const hasIdentity = Boolean(ageIdentityInMemory);
  ageIdEntry.hidden = hasIdentity;
  clearKeyButton.hidden = !hasIdentity;
  updateAgeStorageSettings();
}
function updateAgeStorageSettings() {
  const isStored = Boolean(localStorage.getItem(ageSessionKey));
  ageStorageStatus.dataset.state = isStored ? 'stored' : '';
  ageStorageStatus.textContent = isStored
    ? 'Age private ID is stored in this browser.'
    : ageIdentityInMemory
      ? 'Age private ID is not saved, but is available in this tab’s memory.'
      : 'Age private ID is not stored in this browser.';
  settingsAgeIdInput.hidden = isStored || Boolean(ageIdentityInMemory);
  document.getElementById('settings-age-id-label').hidden = settingsAgeIdInput.hidden;
  saveAgeIdButton.textContent = ageIdentityInMemory ? 'Save current age ID in this browser' : 'Save age ID in this browser';
  saveAgeIdButton.hidden = isStored;
  removeAgeIdButton.hidden = !isStored && !ageIdentityInMemory;
  removeAgeIdButton.textContent = isStored ? 'Remove age ID from this browser' : 'Clear age ID from this tab';
}
function clearAgeIdentity() {
  aiMovies = null;
  aiSeries = null;
  ageIdentityInMemory = '';
  ageInput.value = '';
  rememberAgeId.checked = false;
  localStorage.removeItem(ageSessionKey);
  sessionStorage.removeItem(ageSessionKey);
  updateAgeIdControls();
}
updateAgeIdControls();
rememberAgeId.addEventListener('change', () => {
  if (rememberAgeId.checked && ageIdentityInMemory) localStorage.setItem(ageSessionKey, ageIdentityInMemory);
  else if (!rememberAgeId.checked) localStorage.removeItem(ageSessionKey);
  updateAgeStorageSettings();
});
window.addEventListener('pagehide', () => {
  ageIdentityInMemory = '';
  ageInput.value = '';
  settingsAgeIdInput.value = '';
  metadataExportInput.value = '';
});
window.addEventListener('storage', event => {
  if (event.storageArea !== localStorage || event.key !== ageSessionKey) return;
  if (event.newValue) {
    ageIdentityInMemory = event.newValue;
    rememberAgeId.checked = true;
    updateAgeIdControls();
    contentStatus.textContent = 'Remembered age private ID loaded from this browser.';
    return;
  }
  clearAgeIdentity();
  moviesPanel.replaceChildren();
  resetArrSummary('radarr');
  contentStatus.textContent = 'Age private ID removed from browser storage; metadata is locked.';
});
if (ageIdentityInMemory) contentStatus.textContent = 'Remembered age private ID loaded from this browser. It will be used only with the fossil node you select.';

document.getElementById('age-storage-form').addEventListener('submit', async event => {
  event.preventDefault();
  const identity = ageIdentityInMemory || settingsAgeIdInput.value.trim();
  ageStorageMessage.dataset.state = '';
  if (!identity) {
    ageStorageMessage.dataset.state = 'error';
    ageStorageMessage.textContent = 'Enter an age private ID first.';
    return;
  }
  saveAgeIdButton.disabled = true;
  ageStorageMessage.textContent = 'Validating the age ID with the selected fossil node…';
  try {
    await arrDirectRequest('/settings/age', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ identity, rememberOnDevice: false }),
    });
    ageIdentityInMemory = identity;
    localStorage.setItem(ageSessionKey, identity);
    rememberAgeId.checked = true;
    settingsAgeIdInput.value = '';
    contentStatus.textContent = 'Age private ID saved in this browser and validated by the selected fossil node.';
    ageStorageMessage.textContent = 'Age private ID saved in this browser.';
    updateAgeIdControls();
  } catch (error) {
    ageStorageMessage.dataset.state = 'error';
    ageStorageMessage.textContent = error.message || 'Could not validate or save the age private ID.';
  } finally {
    settingsAgeIdInput.value = '';
    saveAgeIdButton.disabled = false;
  }
});
removeAgeIdButton.addEventListener('click', () => {
  clearAgeIdentity();
  moviesPanel.replaceChildren();
  resetArrSummary('radarr');
  contentStatus.textContent = 'Age private ID removed from this browser; metadata is locked.';
  ageStorageMessage.dataset.state = '';
  ageStorageMessage.textContent = 'Age private ID removed from this browser.';
});

const configurationStatus = document.getElementById('configuration-status');
const configurationJson = document.getElementById('configuration-json');
const secretConfigKey = /apikey|key|token|secret|password|privatekey|credential|authorization|auth|passphrase/i;
function redactConfigSecrets(value) {
  if (Array.isArray(value)) return value.map(redactConfigSecrets);
  if (!value || typeof value !== 'object') return value;
  return Object.fromEntries(Object.entries(value).map(([key, entry]) => [
    key,
    secretConfigKey.test(key.replace(/[_-]/g, '')) && entry !== null && entry !== '' ? '<REDACTED>' : redactConfigSecrets(entry),
  ]));
}
function clearConfigStrings(value) {
  if (Array.isArray(value)) {
    for (let index = 0; index < value.length; index++) {
      if (typeof value[index] === 'string') value[index] = '';
      else clearConfigStrings(value[index]);
    }
  } else if (value && typeof value === 'object') {
    for (const key of Object.keys(value)) {
      if (typeof value[key] === 'string') value[key] = '';
      else clearConfigStrings(value[key]);
    }
  }
}

async function arrDirectUrl() {
  const node = selectedNodeDetails;
  if (!node || node.role !== 'metadata' || !node.online || !node.directUrl) {
    throw new Error('Select an online fossil node with a direct Tailscale route in Devices.');
  }
  const route = await chooseNodeRoute(node, true);
  if (!route.direct) {
    throw new Error('Connection settings require the direct metadata route so the API key does not pass through roarlay.');
  }
  return `${route.base}/api`;
}

async function arrDirectRequest(path, options = {}) {
  const base = await arrDirectUrl();
  const response = await fetch(`${base}${path}`, {
    cache: 'no-store', credentials: 'include', redirect: 'error',
    headers: { Accept: 'application/json', ...(options.headers || {}) }, ...options,
  });
  let result = {};
  try { result = await response.json(); } catch (_) {}
  if (!response.ok) throw new Error(result.error || `Metadata request failed (HTTP ${response.status}).`);
  return result;
}

const reachableArrBases = new Map();
async function probeArrBase(service, url) {
  reachableArrBases.delete(service);
  try {
    const parsed = new URL(url);
    if (parsed.protocol !== 'https:' || parsed.username || parsed.password || parsed.search || parsed.hash) return;
    await fetch(parsed.href, { mode: 'no-cors', credentials: 'omit', cache: 'no-store', referrerPolicy: 'no-referrer', signal: AbortSignal.timeout(3000) });
    reachableArrBases.set(service, parsed.href.replace(/\/$/, ''));
  } catch { /* Keep detail links hidden when the browser cannot reach the Arr service. */ }
}

function appendArrDetailLink(titleCell, service, item) {
  if (titleCell.querySelector('.arr-external-link')) return;
  const base = reachableArrBases.get(service);
  if (!base || !item.titleSlug || ['.', '..'].includes(item.titleSlug)) return;
  const link = document.createElement('a');
  link.className = 'arr-external-link';
  link.href = `${base}/${service === 'radarr' ? 'movie' : 'series'}/${encodeURIComponent(item.titleSlug)}`;
  link.target = '_blank';
  link.rel = 'noopener noreferrer';
  link.title = `Open ${item.title || 'entry'} in ${service === 'radarr' ? 'Radarr' : 'Sonarr'} (new tab)`;
  link.setAttribute('aria-label', link.title);
  link.innerHTML = '<svg viewBox="0 0 16 16" aria-hidden="true" focusable="false"><path d="M6 3H3.75A1.75 1.75 0 0 0 2 4.75v7.5A1.75 1.75 0 0 0 3.75 14h7.5A1.75 1.75 0 0 0 13 12.25V10M8 2h6v6M14 2 7 9"/></svg>';
  titleCell.append(link);
}

function renderLibraryBrowser({ containerId, searchId, filterId, countId, items, type }) {
  const container = document.getElementById(containerId);
  const search = document.getElementById(searchId);
  const filter = document.getElementById(filterId);
  const count = document.getElementById(countId);
  const config = type === 'movie'
    ? { service:'radarr', available:item => item.hasFile, status:item => item.hasFile ? 'Present' : 'Missing', details:item => [item.year || 'Year unknown', item.qualityProfile || 'Quality unknown', item.hasFile ? formatBytes(item.sizeOnDisk) : 'No file'] }
    : { service:'sonarr', art:'TV', available:item => Number(item.episodeFileCount) > 0, status:item => `${Number(item.episodeFileCount) || 0} episodes`, details:item => [item.year || 'Year unknown', `${Number(item.episodeFileCount) || 0} / ${Number(item.episodeCount) || 0} episodes`, item.qualityProfile || 'Quality unknown'] };
  const paint = () => {
    const query = search.value.trim().toLocaleLowerCase();
    const mode = filter.value;
    const visible = items.filter(item => {
      const available = config.available(item);
      if (mode === 'available' && !available || mode === 'missing' && available || mode === 'monitored' && !item.monitored) return false;
      return `${item.title || ''} ${item.year || ''}`.toLocaleLowerCase().includes(query);
    });
    container.replaceChildren();
    count.textContent = `${visible.length.toLocaleString()} of ${items.length.toLocaleString()} ${type === 'movie' ? 'movies' : 'series'}`;
    if (!visible.length) {
      const empty = document.createElement('div'); empty.className = 'library-empty';
      empty.textContent = items.length ? 'No titles match this search or filter.' : `No ${type === 'movie' ? 'movies' : 'series'} in this library yet.`;
      container.append(empty); return;
    }
    for (const item of visible) {
      const card = document.createElement('article'); card.className = 'library-card';
      card.dataset.libraryId = String(type === 'movie' ? item.tmdbId || '' : item.tvdbId || '');
      const art = document.createElement('div'); art.className = 'library-art';
      const placeholder = document.createElement('span'); placeholder.className = 'library-art-empty'; placeholder.textContent = type === 'movie' ? '▶' : '▤'; placeholder.setAttribute('aria-hidden', 'true'); art.append(placeholder);
      const poster = document.createElement('img'); poster.className = 'library-poster'; poster.alt = ''; poster.loading = 'lazy'; poster.decoding = 'async'; poster.referrerPolicy = 'no-referrer';
      if (type === 'movie') { poster.dataset.tmdbId = String(item.tmdbId || ''); if (item.posterUrl) poster.src = item.posterUrl; }
      else if (Number(item.tvdbId) > 0) poster.dataset.tvdbId = String(item.tvdbId);
      poster.addEventListener('load', () => { placeholder.hidden = true; }, { once:true });
      poster.addEventListener('error', () => poster.remove(), { once:true });
      art.append(poster);
      const badge = document.createElement('span'); badge.className = 'library-state'; badge.textContent = config.status(item); art.append(badge);
      const copy = document.createElement('div'); copy.className = 'library-card-copy';
      const titleRow = document.createElement('div'); titleRow.className = 'library-title-row';
      const title = document.createElement('span'); title.className = 'library-card-title'; title.textContent = item.title || 'Untitled'; title.title = item.title || 'Untitled'; titleRow.append(title);
      appendArrDetailLink(titleRow, config.service, item);
      const meta = document.createElement('div'); meta.className = 'library-card-meta'; meta.textContent = `${item.year || 'Year unknown'}${item.monitored ? ' · Monitored' : ''}`;
      const detail = document.createElement('div'); detail.className = 'library-card-detail';
      for (const value of config.details(item)) { const part = document.createElement('span'); part.textContent = value; detail.append(part); }
      copy.append(titleRow, meta, detail); card.append(art, copy); container.append(card);
    }
  };
  search.oninput = paint; filter.onchange = paint; paint();
}

async function unlockMetadataVaultForSettings() {
  const identity = ageIdentityInMemory;
  if (!identity) throw new Error('Enter your age private ID and unlock Movies before saving connection settings.');
  const ageSettings = await arrDirectRequest('/settings/age');
  if (!ageSettings.unlocked) {
    await arrDirectRequest('/settings/age', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ identity, rememberOnDevice: false }),
    });
  }
}

const arrSettings = {
  radarr: {
    copies: [
      { url: 'radarr-url', key: 'radarr-key', form: 'radarr-settings', note: 'radarr-key-note', message: 'radarr-settings-message', button: 'save-radarr', clear: 'clear-radarr', nodeNote: 'radarr-node-note' },
      { url: 'settings-radarr-url', key: 'settings-radarr-key', form: 'settings-radarr-settings', note: 'settings-radarr-key-note', message: 'settings-radarr-settings-message', button: 'settings-save-radarr', clear: 'settings-clear-radarr', nodeNote: 'settings-radarr-node-note' },
    ],
    mediaCard: 'radarr-connection-card', saved: 'settings-radarr-saved', savedUrl: 'settings-radarr-saved-url', endpoint: '/settings/radarr', label: 'Radarr',
  },
  sonarr: {
    copies: [
      { url: 'sonarr-url', key: 'sonarr-key', form: 'sonarr-settings', note: 'sonarr-key-note', message: 'sonarr-settings-message', button: 'save-sonarr', clear: 'clear-sonarr', nodeNote: 'sonarr-node-note' },
      { url: 'settings-sonarr-url', key: 'settings-sonarr-key', form: 'settings-sonarr-settings', note: 'settings-sonarr-key-note', message: 'settings-sonarr-settings-message', button: 'settings-save-sonarr', clear: 'settings-clear-sonarr', nodeNote: 'settings-sonarr-node-note' },
    ],
    mediaCard: 'sonarr-connection-card', saved: 'settings-sonarr-saved', savedUrl: 'settings-sonarr-saved-url', endpoint: '/settings/sonarr', label: 'Sonarr',
  },
};

async function loadArrSettings(service) {
  const fields = arrSettings[service];
  for (const copy of fields.copies) {
    document.getElementById(copy.button).disabled = true;
    const message = document.getElementById(copy.message);
    message.dataset.state = '';
    message.textContent = 'Connecting directly to the selected fossil node…';
  }
  try {
    await unlockMetadataVaultForSettings();
    const settings = await arrDirectRequest(fields.endpoint);
    void probeArrBase(service, settings.url || '');
    document.getElementById(fields.mediaCard).hidden = Boolean(settings.configured);
    for (const copy of fields.copies) {
      document.getElementById(copy.url).value = settings.url || '';
      const key = document.getElementById(copy.key);
      key.value = '';
      key.required = !settings.configured;
      document.getElementById(copy.note).textContent = settings.configured
        ? 'Leave blank to keep the saved API key.'
        : 'Required for the first encrypted save.';
      document.getElementById(copy.nodeNote).textContent = `Settings are sent directly to ${selectedNodeDetails.nodeId}; roarlay does not receive the API key.`;
      const message = document.getElementById(copy.message);
      message.textContent = settings.configured ? `Saved ${fields.label} settings loaded from the age-encrypted config.` : `Enter your ${fields.label} HTTPS URL and API key.`;
      message.dataset.state = '';
      document.getElementById(copy.button).disabled = false;
    }
    const savedCard = document.getElementById(fields.saved);
    savedCard.hidden = !settings.configured;
    document.getElementById(fields.savedUrl).textContent = settings.url || '';
    if (service === 'sonarr') {
      const tvLibrary = document.getElementById('tv-library');
      tvLibrary.hidden = !settings.configured;
      if (settings.configured && (location.hash === '#tv' || location.hash === '#series' || location.hash === '' || location.hash === '#overview')) await loadSonarrSeries();
      if (!settings.configured && (location.hash === '' || location.hash === '#overview')) {
        document.getElementById('overview-series-count').textContent = 'Not connected';
        document.getElementById('overview-series-detail').textContent = 'Connect Sonarr in Settings to see TV statistics.';
      }
    } else if (!settings.configured && (location.hash === '' || location.hash === '#overview')) {
      document.getElementById('overview-movie-count').textContent = 'Not connected';
      document.getElementById('overview-movie-detail').textContent = 'Connect Radarr in Settings to see movie statistics.';
    }
  } catch (error) {
    document.getElementById(fields.saved).hidden = true;
    if (location.hash === '' || location.hash === '#overview') {
      const id = service === 'radarr' ? 'overview-movie-detail' : 'overview-series-detail';
      document.getElementById(id).textContent = error.message || `Could not load ${fields.label} statistics.`;
    }
    for (const copy of fields.copies) {
      const message = document.getElementById(copy.message);
      message.dataset.state = 'error';
      message.textContent = error.message || `Could not load ${fields.label} settings.`;
    }
  } finally {
    for (const copy of fields.copies) document.getElementById(copy.button).disabled = false;
  }
}

for (const [service, fields] of Object.entries(arrSettings)) {
  for (const copy of fields.copies) {
    const form = document.getElementById(copy.form);
    const button = document.getElementById(copy.button);
    const message = document.getElementById(copy.message);
    form.addEventListener('submit', async event => {
      event.preventDefault();
      button.disabled = true;
      message.dataset.state = '';
      message.textContent = `Validating ${fields.label} and saving encrypted settings…`;
      try {
        await unlockMetadataVaultForSettings();
        await arrDirectRequest(fields.endpoint, {
          method: 'PUT', headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ url: document.getElementById(copy.url).value, apiKey: document.getElementById(copy.key).value }),
        });
        for (const peer of fields.copies) document.getElementById(peer.key).value = '';
        await loadArrSettings(service);
        message.textContent = `Connected to ${fields.label}. Settings saved encrypted.`;
      } catch (error) {
        message.dataset.state = 'error';
        message.textContent = error.message || `Could not save ${fields.label} settings.`;
      } finally {
        button.disabled = false;
      }
    });

    document.getElementById(copy.clear).addEventListener('click', async () => {
      try {
        await unlockMetadataVaultForSettings();
        await arrDirectRequest(fields.endpoint, { method: 'DELETE' });
        for (const peer of fields.copies) {
          document.getElementById(peer.url).value = '';
          document.getElementById(peer.key).value = '';
          document.getElementById(peer.key).required = true;
          document.getElementById(peer.note).textContent = 'Required for the first encrypted save.';
          document.getElementById(peer.message).dataset.state = '';
          document.getElementById(peer.message).textContent = `${fields.label} settings cleared from the age-encrypted config.`;
        }
        document.getElementById(fields.mediaCard).hidden = false;
        document.getElementById(fields.saved).hidden = true;
        if (service === 'sonarr') document.getElementById('tv-library').hidden = true;
      } catch (error) {
        message.dataset.state = 'error';
        message.textContent = error.message || `Could not clear ${fields.label} settings.`;
      }
    });
  }
}

async function loadClusterConfiguration() {
  const identity = ageIdentityInMemory;
  if (!identity) {
    configurationStatus.textContent = 'Unlock the age ID in Movies before loading cluster configuration.';
    configurationJson.textContent = 'Cluster configuration is not loaded.';
    return;
  }
  const node = workspaceNodes.find(candidate => candidate.role === 'metadata' && canOpenNode(candidate));
  if (!node) {
    configurationStatus.textContent = 'No reachable fossil node is available to read the shared configuration.';
    return;
  }
  configurationStatus.textContent = 'Fetching encrypted config and decrypting it in this browser…';
  try {
    await import('/assets/metadata-export.js');
    const route = await chooseNodeRoute(node);
    const config = await window.rawrarrMiniExport.loadSharedConfig(identity, route.base);
    configurationJson.textContent = JSON.stringify(redactConfigSecrets(config), null, 2);
    clearConfigStrings(config);
    configurationStatus.textContent = 'Shared configuration loaded; secret values redacted.';
  } catch (error) {
    configurationJson.textContent = 'Cluster configuration could not be loaded.';
    configurationStatus.textContent = error.message || 'Could not decrypt the shared configuration in this browser.';
  }
}
async function jsonRequest(path, options = {}) {
  const response = await fetch(path, { headers: { Accept: 'application/json', ...(options.headers || {}) }, cache: 'no-store', ...options });
  if (!response.ok) {
    if (response.status === 401) {
      clearAgeIdentity();
      throw new Error('Your roarlay sign-in expired. Sign in again.');
    }
    throw new Error(`Request failed (HTTP ${response.status}).`);
  }
  return response.status === 204 ? null : response.json();
}

function openNode(node) {
  if (!node.relayAvailable && !node.directUrl) {
    status.textContent = `${node.nodeId} is approved, but no direct or roarlay route is available.`;
    contentPanel.hidden = true;
    return;
  }
  selectedNode = node.nodeId;
  sessionStorage.setItem(selectedNodeStorageKey, selectedNode);
  selectedNodeDetails = node;
  void loadProfileAvatar();
  selectedRoute = null;
  selectedRoutePromise = null;
  selectedRoutePromiseNode = null;
  const workspaceUrl = new URL(window.location.href);
  workspaceUrl.searchParams.delete('node');
  window.history.replaceState(null, '', workspaceUrl);
  for (const card of nodesPanel.querySelectorAll('.device-card')) {
    card.dataset.selected = String(card.dataset.nodeId === selectedNode);
  }
  contentPanel.hidden = node.role !== 'metadata';
  aiMovies = null;
  aiSeries = null;
  moviesPanel.replaceChildren();
  resetArrSummary('radarr');
  status.textContent = node.role === 'metadata'
    ? `${node.nodeId} · fossil node`
    : `${node.nodeId} · boneyard node · content is available when published by a fossil node`;
  if (node.role === 'metadata') {
    if (location.hash === '' || location.hash === '#overview') void loadOverviewMetadata();
    if (node.preferredDirect) {
      void chooseDirectRoute(node, node.preferredDirect);
    } else {
      void chooseNodeRoute(node);
    }
    if (location.hash === '#movies') void refreshRadarrBackfillStatus();
    if (location.hash === '#tv' || location.hash === '#series') void refreshSonarrBackfillStatus();
    if (ageInput.value.trim() || ageIdentityInMemory) void unlockMetadata();
    const currentView = location.hash.slice(1);
    if (currentView === '' || currentView === '#overview' || currentView === 'overview') {
      void loadArrSettings('radarr');
      void loadArrSettings('sonarr');
    }
    if (currentView === 'movies') void loadArrSettings('radarr');
    if (currentView === 'tv' || currentView === 'series') void loadArrSettings('sonarr');
    if (currentView === 'settings') {
      void loadArrSettings('radarr');
      void loadArrSettings('sonarr');
    }
  }
}

function renderStorageScanStatus(nodeId, message) {
  storageScanStatusByNode.set(nodeId, message);
  const card = [...nodesPanel.querySelectorAll('.device-card')]
    .find(candidate => candidate.dataset.nodeId === nodeId);
  const status = card?.querySelector('.storage-scan-status');
  if (!status) return;
  const statusMessage = status.querySelector('.storage-scan-message');
  if (statusMessage) statusMessage.textContent = message;
  const progress = status.querySelector('.storage-scan-progress');
  if (progress) {
    const running = storageScansActive.has(nodeId);
    progress.hidden = !running;
    progress.setAttribute('aria-valuetext', running ? message : '');
  }
}

function setStorageScanBusy(nodeId, busy) {
  if (busy) storageScansActive.add(nodeId);
  else storageScansActive.delete(nodeId);
  const card = [...nodesPanel.querySelectorAll('.device-card')]
    .find(candidate => candidate.dataset.nodeId === nodeId);
  const progress = card?.querySelector('.storage-scan-progress');
  if (progress) {
    progress.hidden = !busy;
    progress.setAttribute('aria-valuetext', busy ? 'Storage scan is running…' : '');
  }
  for (const button of card?.querySelectorAll('.storage-scan-actions button') || []) {
    button.disabled = busy && !button.classList.contains('storage-scan-stop');
    if (button.classList.contains('storage-scan-stop')) button.hidden = !busy;
  }
}

async function storageScanJson(response) {
  const body = await response.text();
  if (!body.trim()) {
    throw new Error(`Boneyard node returned an empty scan response (HTTP ${response.status}); its scan service may need updating.`);
  }
  try {
    return JSON.parse(body);
  } catch (_) {
    throw new Error(`Boneyard node returned an invalid scan response (HTTP ${response.status}).`);
  }
}

async function storageScanRequest(node, path, options, method = options ? 'POST' : 'GET') {
  const route = await chooseNodeRoute(node);
  if (!route) throw new Error('No route is available to this boneyard node.');
  const send = target => fetch(`${target.base}${path}`, {
    method,
    cache: 'no-store',
    credentials: target.direct ? 'include' : 'same-origin',
    headers: method === 'POST' ? { Accept: 'application/json', 'Content-Type': 'application/json' } : { Accept: 'application/json' },
    ...(method === 'POST' ? { body: JSON.stringify(options || {}) } : {}),
  });
  try {
    let response = await send(route);
    if (route.direct && response.status === 401) {
      const directRoute = deviceRouteStatusByNode.get(node.nodeId)?.directRoute;
      const refreshed = directRoute ? await chooseDirectRoute(node, directRoute) : null;
      if (refreshed?.direct) response = await send(refreshed);
    }
    if (route.direct && !response.ok && response.status !== 400 && response.status !== 409) {
      throw new Error(`Boneyard node request failed (HTTP ${response.status}).`);
    }
    return response;
  } catch (error) {
    if (!route.direct) throw error;
    const fallback = { nodeId: node.nodeId, base: `/nodes/${encodeURIComponent(node.nodeId)}`, direct: false };
    selectedRoute = fallback;
    return send(fallback);
  }
}

async function scanStorageNode(node, options, button) {
  if (!node.online) return;
  openNode(node);
  setStorageScanBusy(node.nodeId, true);
  renderStorageScanStatus(node.nodeId, 'Requesting storage scan…');
  try {
    const started = await storageScanRequest(node, '/api/library/scan/missing', options);
    if (!started.ok && started.status !== 409) {
      throw new Error(`Storage scan request failed (HTTP ${started.status}).`);
    }
    renderStorageScanStatus(node.nodeId, 'Storage scan is running…');
    for (;;) {
      const response = await storageScanRequest(node, '/api/library/scan/status');
      if (!response.ok) throw new Error(`Could not read storage scan status (HTTP ${response.status}).`);
      const result = await storageScanJson(response);
      if (result.error) throw new Error(result.error);
      if (!result.active) {
        renderStorageScanStatus(node.nodeId, result.cancelled
          ? `Scan stopped · ${result.filesConsidered} files checked · ${result.sidecarsCreated} written · ${result.sidecarsSkipped} already complete · ${result.sidecarErrors} errors`
          :
          `Scan complete · ${result.filesConsidered} files checked · ${result.sidecarsCreated} written · ${result.sidecarsSkipped} already complete · ${result.sidecarErrors} errors${result.truncated ? ' · scan limit reached' : ''}`);
        return;
      }
      const completed = Number.isInteger(result.filesCompleted) ? result.filesCompleted : 0;
      const total = Number.isInteger(result.filesTotal) ? result.filesTotal : 0;
      const itemProgress = total > 0 ? ` ${completed} of ${total}` : '';
      renderStorageScanStatus(node.nodeId, `Storage scan is running…${itemProgress}`);
      await new Promise(resolve => setTimeout(resolve, 2000));
    }
  } catch (error) {
    renderStorageScanStatus(node.nodeId, error.message || 'Storage scan failed.');
  } finally {
    setStorageScanBusy(node.nodeId, false);
    button.disabled = false;
  }
}

async function chooseDirectRoute(node, route) {
  if (selectedNode !== node.nodeId) return null;
  try {
    const session = await jsonRequest('/api/workspace/direct-session', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ nodeId: node.nodeId })
    });
    if (new URL(route.base, window.location.href).origin !== window.location.origin) {
      const response = await fetch(`${route.base}/api/direct/session`, {
        method: 'POST', cache: 'no-store', credentials: 'include', redirect: 'error',
        headers: { Authorization: `Bearer ${session.token}` }
      });
      if (!response.ok) throw new Error(`Direct session setup failed (HTTP ${response.status}).`);
    }
    const directRoute = { nodeId: node.nodeId, base: route.base, direct: true };
    if (selectedNode === node.nodeId) {
      selectedRoute = directRoute;
    }
    return directRoute;
  } catch (_) {
    const relayRoute = { nodeId: node.nodeId, base: `/nodes/${encodeURIComponent(node.nodeId)}`, direct: false };
    if (selectedNode === node.nodeId) {
      selectedRoute = relayRoute;
    }
    return relayRoute;
  }
}

async function chooseNodeRoute(node, refresh = false) {
  if (selectedRoute?.nodeId === node.nodeId && (!refresh || !node.directUrl)) return selectedRoute;
  if (selectedRoutePromise && selectedRoutePromiseNode === node.nodeId) return selectedRoutePromise;
  const routePromise = (async () => {
    deviceRouteStatusByNode = await checkDeviceRoutes(workspaceNodes, { refresh });
    const deviceRoute = deviceRouteStatusByNode.get(node.nodeId);
    if (deviceRoute?.status === 'direct') return chooseDirectRoute(node, deviceRoute.directRoute);
    const relayRoute = { nodeId: node.nodeId, base: `/nodes/${encodeURIComponent(node.nodeId)}`, direct: false };
    if (selectedNode === node.nodeId) {
      selectedRoute = relayRoute;
    }
    return relayRoute;
  })();
  selectedRoutePromiseNode = node.nodeId;
  selectedRoutePromise = routePromise;
  try { return await routePromise; }
  finally {
    if (selectedRoutePromise === routePromise) {
      selectedRoutePromise = null;
      selectedRoutePromiseNode = null;
    }
  }
}

function formatBytes(bytes) {
  if (!Number.isFinite(bytes) || bytes < 0) return '—';
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB', 'TB'];
  let value = bytes / 1024;
  for (const unit of units) { if (value < 1024 || unit === 'TB') return `${value.toFixed(1)} ${unit}`; value /= 1024; }
}

function renderGenreBreakdown(elementId, records) {
  const counts = new Map();
  for (const record of records) {
    const uniqueGenres = new Set((Array.isArray(record.genres) ? record.genres : [])
      .filter(genre => typeof genre === 'string' && genre.trim())
      .map(genre => genre.trim()));
    for (const genre of uniqueGenres) counts.set(genre, (counts.get(genre) || 0) + 1);
  }
  const genres = [...counts].sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]));
  const element = document.getElementById(elementId);
  if (!genres.length) {
    element.textContent = 'No genre data available yet.';
    return;
  }

  const palette = ['#a6e3a1', '#89b4fa', '#f9e2af', '#f38ba8', '#cba6f7', '#94e2d5', '#fab387', '#74c7ec', '#eba0ac', '#b4befe', '#9399b2'];
  const topGenres = genres.slice(0, 10);
  const remaining = genres.slice(10);
  const chartGenres = [...topGenres];
  if (remaining.length) chartGenres.push(['Other', remaining.reduce((total, [, count]) => total + count, 0)]);
  const total = chartGenres.reduce((sum, [, count]) => sum + count, 0);
  let cursor = 0;
  const stops = chartGenres.map(([, count], index) => {
    const start = cursor;
    cursor += count / total * 100;
    return `${palette[index]} ${start.toFixed(3)}% ${cursor.toFixed(3)}%`;
  });
  const pie = document.createElement('div');
  pie.className = 'overview-genre-pie';
  pie.setAttribute('role', 'img');
  pie.setAttribute('aria-label', `Genre distribution: ${chartGenres.map(([genre, count]) => `${genre} ${count}`).join(', ')}`);
  pie.style.background = `conic-gradient(${stops.join(', ')})`;
  const legend = document.createElement('div');
  legend.className = 'overview-genre-legend';
  for (const [index, [genre, count]] of chartGenres.entries()) {
    const item = document.createElement('div');
    item.className = 'overview-genre-legend-item';
    const swatch = document.createElement('span');
    swatch.className = 'overview-genre-swatch';
    swatch.style.backgroundColor = palette[index];
    swatch.setAttribute('aria-hidden', 'true');
    const label = document.createElement('span');
    label.textContent = `${genre} ${count}`;
    item.append(swatch, label);
    legend.append(item);
  }
  element.replaceChildren(pie, legend);
}

async function loadOverviewMetadata() {
  const state = document.getElementById('overview-folder-state');
  const detail = document.getElementById('overview-folder-detail');
  const node = selectedNodeDetails;
  if (!node || node.role !== 'metadata') {
    const onlineMetadata = workspaceNodes.some(candidate => candidate.role === 'metadata' && candidate.online);
    state.textContent = onlineMetadata ? 'Fossil node not selected' : 'No online fossil node';
    detail.textContent = onlineMetadata
      ? 'An online fossil node is available in Devices, but it is not selected for this overview.'
      : 'Connect an online fossil node in Devices to see shared-folder details.';
    document.getElementById('overview-metadata-size').textContent = '—';
    return;
  }
  const health = syncthingHealthCache.get(node.nodeId)?.value;
  const folders = health?.folders || [];
  if (!folders.length) {
    state.textContent = 'Unavailable';
    detail.textContent = 'Shared-folder status is not available from this fossil node.';
  } else {
    const complete = folders.filter(folder => folder.state === 'idle' || folder.completionPercent >= 100).length;
    state.textContent = folders.length === 1 ? (folders[0].state || 'Unknown') : `${complete} / ${folders.length} ready`;
    detail.textContent = folders.map(folder => `${folder.folderId}: ${folder.state || 'unknown'}${Number.isFinite(folder.completionPercent) ? ` · ${Math.round(folder.completionPercent)}% synced` : ''}`).join(' · ');
  }
  try {
    const sizes = await jsonRequest(`/nodes/${encodeURIComponent(node.nodeId)}/api/metadata/storage`);
    if (selectedNodeDetails?.nodeId !== node.nodeId) return;
    document.getElementById('overview-metadata-size').textContent = formatBytes(sizes.bytes);
    document.getElementById('overview-metadata-detail').textContent = `${formatBytes(sizes.moviesBytes)} movie metadata · ${formatBytes(Math.max(0, sizes.bytes - sizes.moviesBytes))} other metadata`;
  } catch (_) {
    document.getElementById('overview-metadata-size').textContent = 'Unavailable';
    document.getElementById('overview-metadata-detail').textContent = 'Shared encrypted metadata size could not be read.';
  }
}

let sonarrLoadGeneration = 0;
async function loadSonarrSeries() {
  if (!ageIdentityInMemory) {
    const message = document.getElementById('series-message');
    message.hidden = false;
    message.textContent = 'Enter your age private ID to unlock the TV library in this browser.';
    return;
  }
  const button = document.getElementById('refresh-series');
  if (button.disabled) return;
  button.disabled = true;
  const generation = ++sonarrLoadGeneration;
  const nodeId = selectedNode;
  const message = document.getElementById('series-message');
  const rows = document.getElementById('sonarr-series');
  const cachedSeries = readLibraryTextCache(nodeId, 'tv');
  if (cachedSeries) {
    renderSonarrSeries(cachedSeries, 'Cached TV library. Updating…');
    void attachSonarrPosters(rows);
  }
  message.hidden = false;
  message.dataset.state = '';
  message.textContent = cachedSeries ? 'Cached TV library. Updating…' : 'Loading series from Sonarr…';
  try {
    await unlockMetadataVaultForSettings();
    const series = await arrDirectRequest('/sonarr/series');
    if (generation !== sonarrLoadGeneration || selectedNode !== nodeId) return;
    writeLibraryTextCache(nodeId, 'tv', series);
    renderSonarrSeries(series, `Loaded ${series.length} series from Sonarr.`);
    void attachSonarrPosters(rows);
  } catch (error) {
    if (generation !== sonarrLoadGeneration || selectedNode !== nodeId) return;
    message.hidden = false;
    message.dataset.state = 'error';
    message.textContent = error.message || 'Could not load the Sonarr series library.';
  } finally {
    button.disabled = false;
  }
}

async function attachSonarrPosters(rows) {
  try {
    const artworkBase = await arrDirectUrl();
    for (const poster of rows.querySelectorAll('img[data-tvdb-id]')) {
      if (!poster.isConnected || poster.src) continue;
      poster.src = `${artworkBase}/sonarr/series/${encodeURIComponent(poster.dataset.tvdbId)}/poster`;
      poster.addEventListener('error', () => poster.remove(), { once: true });
    }
  } catch { /* Text rows remain useful when artwork routing is unavailable. */ }
}

function renderSonarrSeries(series, statusMessage = `Loaded ${series.length} series from Sonarr.`) {
    const rows = document.getElementById('sonarr-series');
    aiSeries = series;
    let episodesPresent = 0;
    let diskUsage = 0;
    for (const item of series) {
      const episodeFiles = Number(item.episodeFileCount) || 0;
      const episodeCount = Number(item.episodeCount) || 0;
      const sizeOnDisk = Number(item.sizeOnDisk) || 0;
      episodesPresent += episodeFiles;
      diskUsage += sizeOnDisk;
    }
    renderLibraryBrowser({ containerId:'sonarr-series', searchId:'tv-search', filterId:'tv-filter', countId:'tv-count', items:series, type:'tv' });
    setArrSummaryValues('sonarr', {
      libraryCount: String(series.length),
      filesPresent: String(episodesPresent),
      diskUsage: formatBytes(diskUsage),
    });
    document.getElementById('overview-series-count').textContent = String(series.length);
    document.getElementById('overview-series-detail').textContent = `${episodesPresent.toLocaleString()} episodes · ${formatBytes(diskUsage)} on disk`;
    renderGenreBreakdown('overview-series-genres', series);
    const message = document.getElementById('series-message');
    message.hidden = false;
    message.dataset.state = '';
    message.textContent = statusMessage;
    void refreshSonarrBackfillStatus();
}
document.getElementById('refresh-series').addEventListener('click', async event => {
  const button = event.currentTarget;
  button.disabled = true;
  const message = document.getElementById('series-message');
  message.hidden = false;
  message.dataset.state = '';
  try {
    const nodeId = selectedNodeDetails?.nodeId || 'the selected fossil node';
    message.textContent = `Requesting a TV metadata rescan from ${nodeId}…`;
    await unlockMetadataVaultForSettings();
    await arrDirectRequest('/metadata/sonarr/rescan', { method: 'POST' });
    localStorage.removeItem(libraryTextCacheKey(nodeId, 'tv'));
    message.textContent = `TV metadata rescan started on ${nodeId}.`;
    button.disabled = false;
    await loadSonarrSeries();
  } catch (error) {
    message.dataset.state = 'error';
    message.textContent = error.message || 'Could not start the TV metadata rescan.';
  } finally {
    button.disabled = false;
  }
});

async function refreshSonarrBackfillStatus() {
  await refreshArrScrapeStatus('sonarr');
}

async function refreshRadarrBackfillStatus() {
  await refreshArrScrapeStatus('radarr');
}

async function refreshArrScrapeStatus(provider) {
  const name = provider === 'radarr' ? 'Radarr' : 'Sonarr';
  const label = document.getElementById(`${provider}-scrape-status`);
  const bar = document.getElementById(`${provider}-scrape-progress`);
  try {
    const progress = await arrDirectRequest(provider === 'radarr' ? '/metadata/artwork/backfill' : '/metadata/sonarr/backfill');
    if (!progress.total) { label.hidden = true; bar.hidden = true; return; }
    const complete = progress.scraped >= progress.total;
    const libraryMessage = document.getElementById(provider === 'radarr' ? 'content-status' : 'series-message');
    if (complete) {
      label.hidden = true;
      bar.hidden = true;
      if (provider === 'sonarr' && libraryMessage.dataset.state !== 'error' && libraryMessage.textContent.startsWith('Loaded ')) {
        libraryMessage.hidden = true;
      }
      return;
    }
    label.hidden = false;
    if (provider === 'sonarr' && libraryMessage.dataset.state !== 'error') libraryMessage.hidden = false;
    bar.max = Math.max(1, progress.total);
    bar.value = progress.scraped;
    bar.hidden = !progress.active;
    const saved = `${progress.scraped.toLocaleString()} / ${progress.total.toLocaleString()} posters saved`;
    const pass = progress.active ? ` · ${progress.completed.toLocaleString()} checked this pass` : '';
    const failures = progress.failed ? ` · ${progress.failed.toLocaleString()} unavailable` : '';
    label.textContent = progress.active
      ? `Scraping ${name} metadata and artwork to encrypted shared metadata: ${saved}${pass}${failures}`
      : progress.scraped >= progress.total
        ? `${name} artwork cache complete${failures}`
        : `${name} artwork cache status: ${saved}${failures}`;
  } catch { /* The status will refresh on the next poll. */ }
}

window.setInterval(() => {
  if (location.hash === '#movies' && selectedNodeDetails?.role === 'metadata') void refreshRadarrBackfillStatus();
  if (location.hash === '#tv' || location.hash === '#series') void refreshSonarrBackfillStatus();
}, 3000);

function renderMovies(movies) {
  aiMovies = movies;
  moviesPanel.replaceChildren();
  const files = movies.filter(movie => movie.hasFile).length;
  setArrSummaryValues('radarr', {
    libraryCount: String(movies.length),
    filesPresent: `${files} / ${movies.length}`,
    diskUsage: formatBytes(movies.reduce((total, movie) => total + movie.sizeOnDisk, 0)),
  });
  document.getElementById('overview-movie-count').textContent = String(movies.length);
  document.getElementById('overview-movie-detail').textContent = `${files.toLocaleString()} with files · ${formatBytes(movies.reduce((total, movie) => total + movie.sizeOnDisk, 0))} on disk`;
  renderGenreBreakdown('overview-movie-genres', movies);
  void loadMetadataSizes();
  void arrDirectRequest('/settings/radarr').then(async settings => {
    await probeArrBase('radarr', settings.url || '');
    for (const card of moviesPanel.querySelectorAll('.library-card')) {
      const title = card.querySelector('.library-card-title');
      const movie = movies.find(candidate => candidate.title === title?.textContent);
      if (movie) appendArrDetailLink(card.querySelector('.library-title-row'), 'radarr', movie);
    }
  }).catch(() => {});
  renderLibraryBrowser({ containerId:'movies', searchId:'movies-search', filterId:'movies-filter', countId:'movies-count', items:movies, type:'movie' });
}

function renderCachedMovies(nodeId) {
  if (!ageIdentityInMemory) return false;
  const cached = readLibraryTextCache(nodeId, 'movies');
  if (!cached) return false;
  renderMovies(cached.map(movie => ({ ...movie, posterUrl: null })));
  contentStatus.textContent = 'Showing cached movie library while updating…';
  void attachMoviePosters(cached, nodeId);
  return true;
}

async function loadMetadataSizes() {
  const node = selectedNodeDetails;
  if (!node) return;
  try {
    const sizes = await jsonRequest(`/nodes/${encodeURIComponent(node.nodeId)}/api/metadata/storage`);
    setArrSummaryValues('radarr', { metadataSize: formatBytes(sizes.moviesBytes), allMetadataSize: formatBytes(sizes.bytes) });
    setArrSummaryValues('sonarr', { metadataSize: formatBytes(sizes.tvBytes), allMetadataSize: formatBytes(sizes.bytes) });
  } catch (_) {
    setArrSummaryValues('radarr', { metadataSize: '—', allMetadataSize: '—' });
    setArrSummaryValues('sonarr', { metadataSize: '—', allMetadataSize: '—' });
  }
}

async function unlockMetadata() {
  const suppliedIdentity = ageInput.value.trim();
  const identity = suppliedIdentity || ageIdentityInMemory;
  if (!selectedNode || !identity) return;
  const nodeId = selectedNode;
  const node = selectedNodeDetails;
  if (!node || node.nodeId !== nodeId) return;
  const hasCachedMovies = renderCachedMovies(nodeId);
  ageIdentityInMemory = identity;
  if (rememberAgeId.checked) localStorage.setItem(ageSessionKey, identity);
  else localStorage.removeItem(ageSessionKey);
  ageInput.value = '';
  updateAgeIdControls();
  if (location.hash === '#movies') void loadArrSettings('radarr');
  if (location.hash === '#tv' || location.hash === '#series') void loadArrSettings('sonarr');
  contentStatus.textContent = hasCachedMovies ? 'Showing cached movie library while updating…' : 'Connecting to Radarr through the node direct route…';
  try {
    await unlockMetadataVaultForSettings();
    const liveMovies = await arrDirectRequest('/radarr/live-library');
    if (selectedNode !== nodeId) return;
    const movies = liveMovies.map(movie => ({
      ...movie,
      qualityProfile: movie.qualityProfile || '—',
      sizeOnDisk: Number(movie.sizeOnDisk) || 0,
      posterUrl: null,
    }));
    if (selectedNode !== nodeId) return;
    writeLibraryTextCache(nodeId, 'movies', movies.map(({ posterUrl: _posterUrl, ...movie }) => movie));
    renderMovies(movies);
    void attachMoviePosters(movies, nodeId);
    contentStatus.textContent = `${movies.length} Radarr library entries loaded directly from the node.`;
  } catch (error) {
    if (selectedNode !== nodeId) return;
    if (!hasCachedMovies) moviesPanel.replaceChildren();
    updateAgeIdControls();
    contentStatus.textContent = error.message || 'Could not decrypt metadata in this browser.';
  }
}

async function attachMoviePosters(movies, nodeId) {
  try {
    const artworkBase = await arrDirectUrl();
    if (selectedNode !== nodeId) return;
    for (const image of moviesPanel.querySelectorAll('img[data-tmdb-id]')) {
      const movie = movies.find(candidate => String(candidate.tmdbId) === image.dataset.tmdbId);
      if (!movie?.tmdbId || !movie.artworkSource || image.src || !image.isConnected) continue;
      image.src = `${artworkBase}/radarr/live-movies/${encodeURIComponent(movie.tmdbId)}/poster`;
      image.addEventListener('error', () => image.remove(), { once: true });
    }
  } catch { /* Text rows remain useful when artwork routing is unavailable. */ }
}

document.getElementById('unlock').addEventListener('click', () => { void unlockMetadata(); });
document.getElementById('refresh-movies').addEventListener('click', async event => {
  const button = event.currentTarget;
  button.disabled = true;
  const nodeId = selectedNodeDetails?.nodeId || 'the selected fossil node';
  contentStatus.textContent = `Requesting a movie metadata rescan from ${nodeId}…`;
  try {
    await unlockMetadataVaultForSettings();
    await arrDirectRequest('/metadata/artwork/rescan', { method: 'POST' });
    localStorage.removeItem(libraryTextCacheKey(nodeId, 'movies'));
    contentStatus.textContent = `Movie metadata rescan started on ${nodeId}.`;
    await unlockMetadata();
  } catch (error) {
    contentStatus.textContent = error.message || 'Could not start the movie metadata rescan.';
  } finally {
    button.disabled = false;
  }
});
ageInput.addEventListener('change', () => { if (selectedNode) void unlockMetadata(); });
clearKeyButton.addEventListener('click', () => {
  clearAgeIdentity(); moviesPanel.replaceChildren();
  updateAgeIdControls();
  resetArrSummary('radarr');
  contentStatus.textContent = 'Private ID cleared from this browser.';
});

async function loadNodes() {
  if (savingDeviceName || document.activeElement?.classList.contains('device-name-input')) return;
  try {
    const nodes = await jsonRequest('/api/workspace/nodes');
    workspaceNodes = nodes;
    void loadProfileAvatar();
    const healthByNode = new Map(await Promise.all(nodes.map(async node => {
      if (!node.online || !node.relayAvailable) return [node.nodeId, null];
      const cached = syncthingHealthCache.get(node.nodeId);
      if (cached && Date.now() - cached.checkedAt < 30000) return [node.nodeId, cached.value];
      try {
        const value = await jsonRequest(`/nodes/${encodeURIComponent(node.nodeId)}/api/syncthing/shared-health`);
        syncthingHealthCache.set(node.nodeId, { checkedAt: Date.now(), value });
        return [node.nodeId, value];
      } catch (_) {
        syncthingHealthCache.set(node.nodeId, { checkedAt: Date.now(), value: null });
        return [node.nodeId, null];
      }
    })));
    if (location.hash === '' || location.hash === '#overview') void loadOverviewMetadata();
    deviceRouteStatusByNode = await checkDeviceRoutes(nodes);
    const observedSyncthingPeers = nodes.flatMap(source => {
      const report = healthByNode.get(source.nodeId);
      return (report?.folders || []).flatMap(folder => {
        const peers = folder.peers || [];
        return peers.map(peer => ({
          ...peer,
          folderId: folder.folderId,
          sourceNodeId: source.nodeId,
          folderPeerCount: peers.length,
          folderConnectedPeerCount: peers.filter(candidate => candidate.connected).length,
        }));
      });
    });
    const ageIdentity = ageIdentityInMemory;
    const metadataNodes = nodes.filter(node => node.role === 'metadata' && (node.relayAvailable || node.directUrl));
    const configNode = metadataNodes.find(node => node.online && node.directUrl)
      || metadataNodes.find(node => node.online)
      || metadataNodes[0];
    if (ageIdentity && configNode) {
      try {
        await import('/assets/metadata-export.js');
        deviceNames = await window.rawrarrMiniExport.loadDeviceNames(ageIdentity, `/nodes/${encodeURIComponent(configNode.nodeId)}`);
      } catch (_) { /* Names remain editable when the encrypted config is unavailable. */ }
    }
    nodesPanel.replaceChildren();
    if (nodes.length === 0) {
      renderDeviceTopology([], [], {});
      status.textContent = 'No fossil or boneyard nodes are connected to this account yet.';
      contentPanel.hidden = true;
      return;
    }
    let defaultNode = null;
    for (const node of nodes) {
      const card = document.createElement('div');
      card.className = 'device-card';
      card.dataset.nodeId = node.nodeId;
      card.dataset.selected = String(node.nodeId === selectedNode);
      card.tabIndex = 0;
      card.setAttribute('role', 'group');
      card.setAttribute('aria-label', `${roleLabel(node.role)} device`);
      const state = document.createElement('span');
      const route = deviceRouteStatusByNode.get(node.nodeId);
      const routeLabel = {
        direct: 'Direct',
        relay: 'Relay',
        offline: 'Offline',
      }[route?.status || 'offline'];
      const latencyLabel = route?.status === 'direct' && route.directRoute
        ? `${route.directRoute.latencyMs.toFixed(0)} ms`
        : '';
      const routeState = routeLabel.toLowerCase();
      state.className = `node-state ${routeState}`;
      state.setAttribute('aria-label', route?.status === 'direct' && latencyLabel
        ? `Direct Tailscale route active (${latencyLabel}).`
        : routeLabel);
      const indicator = document.createElement('span');
      indicator.className = 'node-state-indicator';
      indicator.setAttribute('aria-hidden', 'true');
      const stateLabel = document.createElement('span');
      stateLabel.className = 'node-state-label';
      stateLabel.textContent = latencyLabel ? `${routeLabel} · ${latencyLabel}` : routeLabel;
      state.append(indicator, stateLabel);
      const nameInput = document.createElement('input');
      nameInput.className = 'device-name-input';
      nameInput.type = 'text';
      nameInput.value = deviceNames[node.nodeId] || node.displayName || node.nodeId;
      nameInput.maxLength = 80;
      nameInput.setAttribute('aria-label', `Name for ${node.nodeId}`);
      nameInput.title = 'Device name';
      nameInput.readOnly = true;
      nameInput.disabled = !ageIdentity || !configNode;
      const editNameButton = document.createElement('button');
      editNameButton.type = 'button';
      editNameButton.className = 'device-name-edit-button';
      editNameButton.textContent = 'Edit';
      editNameButton.setAttribute('aria-label', `Edit name for ${node.nodeId}`);
      editNameButton.disabled = !ageIdentity || !configNode;
      editNameButton.title = ageIdentity && configNode ? 'Edit this device name.' : 'Unlock the age identity to edit device names.';
      editNameButton.addEventListener('click', event => {
        event.stopPropagation();
        nameInput.readOnly = false;
        nameInput.focus();
        nameInput.select();
      });
      const heading = document.createElement('div');
      heading.className = 'device-heading';
      const nameGroup = document.createElement('div');
      nameGroup.className = 'device-name-group';
      nameGroup.append(nameInput, editNameButton);
      heading.append(nameGroup, state);
      const detail = document.createElement('small');
      detail.className = 'device-detail';
      detail.textContent = `${roleLabel(node.role)}${node.online ? '' : ' · last seen earlier'}`;
      const avatarPicture = document.createElement('picture');
      avatarPicture.className = 'device-avatar';
      const avatarSource = document.createElement('source');
      avatarSource.type = 'image/jxl';
      const avatar = document.createElement('img');
      avatar.alt = '';
      avatar.setAttribute('aria-hidden', 'true');
      avatarPicture.append(avatarSource, avatar);
      const identityRow = document.createElement('div');
      identityRow.className = 'device-identity-row';
      identityRow.append(avatarPicture, heading);
      card.append(identityRow, detail);
      void loadNodeAvatar(node, avatarSource, avatar);
      const deleteButton = document.createElement('button');
      deleteButton.type = 'button';
      deleteButton.className = 'device-delete-button';
      deleteButton.textContent = 'Delete device';
      deleteButton.setAttribute('aria-label', `Delete ${deviceNames[node.nodeId] || node.displayName || node.nodeId}`);
      deleteButton.addEventListener('click', async event => {
        event.stopPropagation();
        const displayName = deviceNames[node.nodeId] || node.displayName || node.nodeId;
        if (!window.confirm(`Remove ${displayName} from Rawrarr? This revokes coordinator access but does not delete data on the device.`)) return;
        deleteButton.disabled = true;
        try {
          await jsonRequest(`/api/workspace/nodes/${encodeURIComponent(node.nodeId)}`, { method: 'DELETE' });
          if (selectedNode === node.nodeId) {
            selectedNode = null;
            selectedNodeDetails = null;
            selectedRoute = null;
            contentPanel.hidden = true;
          }
          delete deviceNames[node.nodeId];
          await loadNodes();
        } catch (error) {
          deleteButton.disabled = false;
          status.textContent = error.message || 'Could not delete this device.';
        }
      });
      const syncHealth = document.createElement('small');
      syncHealth.className = 'device-detail device-sync-health';
      const setSyncHealth = parts => {
        syncHealth.replaceChildren(...parts.flatMap((part, index) => {
          const label = document.createElement('span');
          label.textContent = part;
          label.className = index === 0 ? 'sync-health-state' : 'sync-health-detail';
          return index ? [document.createTextNode(' · '), label] : [label];
        }));
      };
      const report = healthByNode.get(node.nodeId);
      const identityTokens = new Set([...deviceNameTokens(node.nodeId), ...deviceNameTokens(deviceNames[node.nodeId])]);
      const matchingPeers = observedSyncthingPeers.filter(peer => {
        const peerTokens = deviceNameTokens(peer.name);
        return [...identityTokens].some(token => peerTokens.has(token));
      });
      if (matchingPeers.length === 1) {
        const peer = matchingPeers[0];
        const state = !peer.connected ? 'disconnected' : Number.isFinite(peer.completionPercent) ? (peer.completionPercent >= 100 ? 'synced' : 'syncing') : 'connected · status unknown';
        setSyncHealth([
          state,
          ...(Number.isFinite(peer.completionPercent) ? [`${Math.round(peer.completionPercent)}% complete`] : []),
          `${peer.folderConnectedPeerCount}/${peer.folderPeerCount} peers connected`,
        ]);
      } else if (report?.configured && report.folders?.length) {
        const details = report.folders.map(folder => {
          const peers = folder.peers || [];
          const parts = [folder.state];
          if (Number.isFinite(folder.completionPercent)) parts.push(`${Math.round(folder.completionPercent)}% complete`);
          parts.push(`${peers.filter(peer => peer.connected).length}/${peers.length} peers connected`);
          return parts;
        });
        setSyncHealth(details.flat());
      } else if (!node.online) {
        setSyncHealth(['Node offline']);
      } else if (!report?.configured) {
        setSyncHealth(['Syncthing health unavailable']);
      } else {
        setSyncHealth(['No shared folders configured']);
      }
      card.append(syncHealth);
      card.append(deleteButton);
      if (node.role === 'storage') {
        const scanActions = document.createElement('div');
        scanActions.className = 'storage-scan-actions';
        const actions = [
          ['Scan, hash & ffprobe media', {}, false],
          ['Scan, hash & ffprobe 3 movies (1–2 GB)', {
            kind: 'movie',
            minimumSizeBytes: 1_000_000_000,
            maximumSizeBytes: 2_000_000_000,
            maxFiles: 3,
          }, true],
        ];
        for (const [label, options] of actions) {
          const button = document.createElement('button');
          button.type = 'button';
          button.textContent = label;
          button.disabled = !node.online || storageScansActive.has(node.nodeId);
          button.addEventListener('click', event => {
            event.stopPropagation();
            void scanStorageNode(node, options, button);
          });
          scanActions.append(button);
        }
        const stopButton = document.createElement('button');
        stopButton.type = 'button';
        stopButton.className = 'storage-scan-stop';
        stopButton.textContent = 'Stop scan';
        stopButton.hidden = !storageScansActive.has(node.nodeId);
        stopButton.disabled = !storageScansActive.has(node.nodeId);
        stopButton.addEventListener('click', async event => {
          event.stopPropagation();
          stopButton.disabled = true;
          try {
            const response = await storageScanRequest(node, '/api/library/scan/cancel', {}, 'POST');
            if (!response.ok) throw new Error(`Could not stop storage scan (HTTP ${response.status}).`);
            renderStorageScanStatus(node.nodeId, 'Stopping after the current file…');
          } catch (error) {
            stopButton.disabled = false;
            renderStorageScanStatus(node.nodeId, error.message || 'Could not stop storage scan.');
          }
        });
        scanActions.append(stopButton);
        const scanStatus = document.createElement('div');
        scanStatus.className = 'storage-scan-status';
        scanStatus.setAttribute('role', 'status');
        scanStatus.setAttribute('aria-live', 'polite');
        const scanMessage = document.createElement('span');
        scanMessage.className = 'storage-scan-message';
        scanMessage.textContent = storageScanStatusByNode.get(node.nodeId) || 'Reads media and writes encrypted sidecars to shared storage.';
        const scanProgress = document.createElement('div');
        scanProgress.className = 'storage-scan-progress';
        scanProgress.setAttribute('role', 'progressbar');
        scanProgress.setAttribute('aria-label', 'Storage scan progress');
        scanProgress.setAttribute('aria-valuetext', 'Storage scan is running…');
        scanProgress.hidden = !storageScansActive.has(node.nodeId);
        const scanProgressIndicator = document.createElement('div');
        scanProgressIndicator.className = 'storage-scan-progress-indicator';
        scanProgress.append(scanProgressIndicator);
        scanStatus.append(scanMessage, scanProgress);
        card.append(scanActions, scanStatus);
      }
      card.addEventListener('click', event => {
        if (event.target !== nameInput) openNode(node);
      });
      card.addEventListener('keydown', event => {
        if (event.target === card && (event.key === 'Enter' || event.key === ' ')) {
          event.preventDefault();
          openNode(node);
        }
      });
      nameInput.addEventListener('keydown', event => {
        if (event.key === 'Enter') { event.preventDefault(); nameInput.blur(); }
        if (event.key === 'Escape') {
          nameInput.value = deviceNames[node.nodeId] || node.displayName || node.nodeId;
          nameInput.readOnly = true;
          nameInput.blur();
        }
      });
      nameInput.addEventListener('change', async () => {
        nameInput.readOnly = true;
        const cleaned = nameInput.value.trim();
        if (!cleaned || cleaned.length > 80 || /[\u0000-\u001f\u007f]/.test(cleaned)) {
          nameInput.value = deviceNames[node.nodeId] || node.displayName || node.nodeId;
          document.getElementById('device-name-status').textContent = 'Enter a name of 1 to 80 printable characters.';
          return;
        }
        const stored = deviceNames[node.nodeId] || '';
        if (cleaned === (stored || node.nodeId)) return;
        nameInput.disabled = true;
        savingDeviceName = true;
        document.getElementById('device-name-status').textContent = 'Saving device name into the age-encrypted config…';
        try {
          await window.rawrarrMiniExport.saveDeviceName(ageIdentity, `/nodes/${encodeURIComponent(configNode.nodeId)}`, node.nodeId, cleaned === node.nodeId ? '' : cleaned);
          deviceNames[node.nodeId] = cleaned === node.nodeId ? '' : cleaned;
          document.getElementById('device-name-status').textContent = 'Device name saved in the shared age-encrypted config.';
          savingDeviceName = false;
          await loadNodes();
        } catch (error) {
          savingDeviceName = false;
          document.getElementById('device-name-status').textContent = error.message || 'Could not save this device name.';
          nameInput.disabled = false;
          nameInput.readOnly = false;
          nameInput.focus();
        }
      });
      nodesPanel.append(card);
      if (node.role === 'metadata' && (node.relayAvailable || node.directUrl) && node.online) {
        if (!defaultNode) defaultNode = node;
      }
    }
    renderDeviceTopology(nodes, observedSyncthingPeers, deviceNames);
    status.textContent = 'Choose a connected fossil or boneyard node to open its Rawrarr page.';
    const canOpen = canOpenNode;
    const requested = requestedNode && nodes.find(node => node.nodeId === requestedNode && canOpen(node));
    // Node status polling must not reset the active workspace or refetch its
    // decrypted library. Only select a node on initial load or when selection
    // becomes invalid; status-only changes are reflected by the rebuilt buttons.
    if (selectedNode) {
      const current = nodes.find(node => node.nodeId === selectedNode && canOpen(node));
      if (current) {
        selectedNodeDetails = current;
        for (const card of nodesPanel.querySelectorAll('.device-card')) {
          card.dataset.selected = String(card.dataset.nodeId === selectedNode);
        }
      } else {
        selectedNode = null;
        selectedNodeDetails = null;
        selectedRoute = null;
        contentPanel.hidden = true;
      }
    }
    if (selectedNode && selectedNodeDetails?.role !== 'metadata' && defaultNode) {
      // Boneyard nodes can be opened explicitly, but they cannot provide the
      // coordinator workspace data shown on Overview, Movies, and TV.
      selectedNode = null;
      selectedNodeDetails = null;
      selectedRoute = null;
      contentPanel.hidden = true;
      const directCandidates = nodes
        .filter(node => node.role === 'metadata' && node.online)
        .map(node => ({ node, route: deviceRouteStatusByNode.get(node.nodeId)?.directRoute }))
        .filter(candidate => candidate.route);
      if (directCandidates.length) {
        const fastest = directCandidates.sort((a, b) => a.route.latencyMs - b.route.latencyMs)[0];
        fastest.node.preferredDirect = fastest.route;
        openNode(fastest.node);
      } else {
        openNode(defaultNode);
      }
    } else if (!selectedNode && requested) {
      openNode(requested);
    } else if (!selectedNode && defaultNode) {
      const directCandidates = nodes
        .filter(node => node.role === 'metadata')
        .map(node => ({ node, route: deviceRouteStatusByNode.get(node.nodeId)?.directRoute }))
        .filter(candidate => candidate.route);
      if (directCandidates.length) {
        const fastest = directCandidates.sort((a, b) => a.route.latencyMs - b.route.latencyMs)[0];
        fastest.node.preferredDirect = fastest.route;
        openNode(fastest.node);
      } else {
        openNode(defaultNode);
      }
    }
  } catch (error) {
    status.textContent = error.message || 'Could not load connected nodes.';
  }
}

async function loadPending() {
  try {
    const pending = await jsonRequest('/api/workspace/pending');
    pendingPanel.hidden = pending.length === 0;
    pendingPanel.classList.toggle('has-pending', pending.length > 0);
    pendingPanel.replaceChildren();
    if (!pending.length) return;
    const heading = document.createElement('div');
    heading.className = 'pending-heading';
    const title = document.createElement('strong');
    title.textContent = 'Nodes waiting for approval';
    const badge = document.createElement('span');
    badge.className = 'pending-badge';
    badge.textContent = 'Needs review';
    heading.append(title, badge);
    pendingPanel.append(heading);
    for (const node of pending) {
      const item = document.createElement('div');
      item.className = 'pending-item';
      const label = document.createElement('span');
      label.textContent = `${node.displayName} · ${roleLabel(node.role)} · joins ${node.ownerDisplayName}`;
      const approve = document.createElement('button');
      approve.type = 'button';
      approve.textContent = 'Approve';
      approve.addEventListener('click', async () => {
        approve.disabled = true;
        try {
          await jsonRequest('/api/workspace/approve', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ requestId: node.requestId }) });
          await loadPending();
          await loadNodes();
        } catch (error) {
          approve.disabled = false;
          status.textContent = error.message || 'Could not approve this node.';
        }
      });
      const reject = document.createElement('button');
      reject.type = 'button';
      reject.className = 'reject-button';
      reject.textContent = 'Reject';
      reject.addEventListener('click', async () => {
        approve.disabled = true;
        reject.disabled = true;
        try {
          await jsonRequest('/api/workspace/reject', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ requestId: node.requestId }) });
          await loadPending();
        } catch (error) {
          approve.disabled = false;
          reject.disabled = false;
          status.textContent = error.message || 'Could not reject this node request.';
        }
      });
      const actions = document.createElement('div');
      actions.className = 'pending-actions';
      actions.append(approve, reject);
      item.append(label, actions);
      pendingPanel.append(item);
    }
  } catch (_) {
    pendingPanel.hidden = true;
  }
}

document.getElementById('sign-out').addEventListener('click', async () => {
  clearAgeIdentity();
  try { await jsonRequest('/api/auth/logout', { method: 'POST' }); } catch (_) {}
  window.location.replace('/auth');
});

await loadNodes();
await loadPending();
if (location.hash === '#configuration') void loadClusterConfiguration();
if (location.hash === '#movies') void loadArrSettings('radarr');
if (location.hash === '#tv' || location.hash === '#series') void loadArrSettings('sonarr');
setInterval(() => { void loadNodes(); void loadPending(); }, 10000);
