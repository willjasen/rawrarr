import { checkDeviceRoutes } from './coordinator-device-routes.mjs';
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
  return role === 'coordinator' ? 'roarlay' : role === 'storage' ? 'apex' : 'metadata';
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
    if (ageIdentityInMemory && selectedNode) void unlockMetadata();
  }
  if (view === 'series' && updateHash) {
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
  const name = words.join('');
  const profileName = document.getElementById('profile-name');
  profileName.replaceChildren(...[
    document.createTextNode(words.slice(0, 2).join(' ')),
    document.createElement('br'),
    document.createTextNode(words.slice(2).join(' ')),
  ]);
  document.getElementById('sidebar-profile-name').textContent = name;
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
    if (!response.ok) throw new Error('Public age ID is not available from this metadata node.');
    const recipient = (await response.text()).trim();
    if (!/^age1pq1[a-z0-9]+$/.test(recipient)) throw new Error('Metadata node returned an invalid public age ID.');
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
      const tokens = new Set([...deviceNameTokens(node.nodeId), ...deviceNameTokens(deviceNames[node.nodeId])]);
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
    name.textContent = deviceNames[node.nodeId] || node.nodeId;
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
    metadataExportStatus.textContent = 'Select a metadata node before exporting.';
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
if (ageIdentityInMemory) contentStatus.textContent = 'Remembered age private ID loaded from this browser. It will be used only with the metadata node you select.';

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
  ageStorageMessage.textContent = 'Validating the age ID with the selected metadata node…';
  try {
    await arrDirectRequest('/settings/age', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ identity, rememberOnDevice: false }),
    });
    ageIdentityInMemory = identity;
    localStorage.setItem(ageSessionKey, identity);
    rememberAgeId.checked = true;
    settingsAgeIdInput.value = '';
    contentStatus.textContent = 'Age private ID saved in this browser and validated by the selected metadata node.';
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
    throw new Error('Select an online metadata node with a direct Tailscale route in Devices.');
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
    message.textContent = 'Connecting directly to the selected metadata node…';
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
    configurationStatus.textContent = 'No reachable metadata node is available to read the shared configuration.';
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
    ? `${node.nodeId} · metadata node`
    : `${node.nodeId} · apex node · content is available when published by a metadata node`;
  if (node.role === 'metadata') {
    if (location.hash === '' || location.hash === '#overview') void loadOverviewMetadata();
    if (node.preferredDirect) {
      void chooseDirectRoute(node, node.preferredDirect);
    } else {
      void chooseNodeRoute(node);
    }
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
    throw new Error(`Storage node returned an empty scan response (HTTP ${response.status}); its scan service may need updating.`);
  }
  try {
    return JSON.parse(body);
  } catch (_) {
    throw new Error(`Storage node returned an invalid scan response (HTTP ${response.status}).`);
  }
}

async function storageScanRequest(node, path, options, method = options ? 'POST' : 'GET') {
  const route = await chooseNodeRoute(node);
  if (!route) throw new Error('No route is available to this storage node.');
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
      throw new Error(`Storage node request failed (HTTP ${response.status}).`);
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
    state.textContent = onlineMetadata ? 'Metadata node not selected' : 'No online metadata node';
    detail.textContent = onlineMetadata
      ? 'An online metadata node is available in Devices, but it is not selected for this overview.'
      : 'Connect an online metadata node in Devices to see shared-folder details.';
    document.getElementById('overview-metadata-size').textContent = '—';
    return;
  }
  const health = syncthingHealthCache.get(node.nodeId)?.value;
  const folders = health?.folders || [];
  if (!folders.length) {
    state.textContent = 'Unavailable';
    detail.textContent = 'Shared-folder status is not available from this metadata node.';
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
  const button = document.getElementById('refresh-series');
  if (button.disabled) return;
  button.disabled = true;
  const generation = ++sonarrLoadGeneration;
  const nodeId = selectedNode;
  const message = document.getElementById('series-message');
  const rows = document.getElementById('sonarr-series');
  message.hidden = false;
  message.dataset.state = '';
  message.textContent = 'Loading series from Sonarr…';
  try {
    await unlockMetadataVaultForSettings();
    const series = await arrDirectRequest('/sonarr/series');
    const sonarrSettings = await arrDirectRequest('/settings/sonarr');
    await probeArrBase('sonarr', sonarrSettings.url || '');
    if (generation !== sonarrLoadGeneration || selectedNode !== nodeId) return;
    const artworkBase = await arrDirectUrl();
    if (generation !== sonarrLoadGeneration || selectedNode !== nodeId) return;
    aiSeries = series;
    rows.replaceChildren();
    let episodesPresent = 0;
    let diskUsage = 0;
    for (const item of series) {
      const episodeFiles = Number(item.episodeFileCount) || 0;
      const episodeCount = Number(item.episodeCount) || 0;
      const sizeOnDisk = Number(item.sizeOnDisk) || 0;
      episodesPresent += episodeFiles;
      diskUsage += sizeOnDisk;
      const row = document.createElement('tr');
      const title = document.createElement('td'); title.className = 'title-cell';
      const titleContent = document.createElement('span'); titleContent.className = 'title-content';
      if (Number(item.tvdbId) > 0) {
        const poster = document.createElement('img');
        poster.className = 'poster'; poster.alt = ''; poster.loading = 'lazy';
        poster.decoding = 'async'; poster.referrerPolicy = 'no-referrer';
        poster.src = `${artworkBase}/sonarr/series/${encodeURIComponent(item.tvdbId)}/poster`;
        poster.addEventListener('error', () => poster.remove(), { once: true });
        titleContent.append(poster);
      }
      const titleText = document.createElement('span');
      titleText.textContent = `${item.title || 'Untitled'}${item.year ? ` (${item.year})` : ''}`;
      titleContent.append(titleText);
      title.append(titleContent);
      appendArrDetailLink(title, 'sonarr', item);
      const tvdbId = document.createElement('td'); tvdbId.textContent = item.tvdbId ? String(item.tvdbId) : '—';
      const year = document.createElement('td'); year.textContent = item.year ? String(item.year) : '—';
      const episodes = document.createElement('td'); episodes.textContent = `${episodeFiles} / ${episodeCount}`;
      const monitored = document.createElement('td'); monitored.textContent = item.monitored ? 'Yes' : 'No';
      const quality = document.createElement('td'); quality.textContent = item.qualityProfile || '—';
      const size = document.createElement('td'); size.textContent = sizeOnDisk ? formatBytes(sizeOnDisk) : '—';
      row.append(title, tvdbId, year, episodes, monitored, quality, size);
      rows.append(row);
    }
    setArrSummaryValues('sonarr', {
      libraryCount: String(series.length),
      filesPresent: String(episodesPresent),
      diskUsage: formatBytes(diskUsage),
    });
    document.getElementById('overview-series-count').textContent = String(series.length);
    document.getElementById('overview-series-detail').textContent = `${episodesPresent.toLocaleString()} episodes · ${formatBytes(diskUsage)} on disk`;
    renderGenreBreakdown('overview-series-genres', series);
    message.hidden = false;
    message.textContent = `Loaded ${series.length} series from Sonarr.`;
    void refreshSonarrBackfillStatus();
  } catch (error) {
    if (generation !== sonarrLoadGeneration || selectedNode !== nodeId) return;
    message.hidden = false;
    message.dataset.state = 'error';
    message.textContent = error.message || 'Could not load the Sonarr series library.';
  } finally {
    button.disabled = false;
  }
}
document.getElementById('refresh-series').addEventListener('click', async event => {
  const button = event.currentTarget;
  button.disabled = true;
  const message = document.getElementById('series-message');
  message.hidden = false;
  message.dataset.state = '';
  try {
    const nodeId = selectedNodeDetails?.nodeId || 'the selected metadata node';
    message.textContent = `Requesting a TV metadata rescan from ${nodeId}…`;
    await unlockMetadataVaultForSettings();
    await arrDirectRequest('/metadata/sonarr/rescan', { method: 'POST' });
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
  try {
    const progress = await arrDirectRequest('/metadata/sonarr/backfill');
    const label = document.getElementById('sonarr-scrape-status');
    const bar = document.getElementById('sonarr-scrape-progress');
    if (!progress.total) { label.hidden = true; bar.hidden = true; return; }
    const complete = progress.scraped >= progress.total;
    const seriesMessage = document.getElementById('series-message');
    if (complete) {
      label.hidden = true;
      bar.hidden = true;
      if (seriesMessage.dataset.state !== 'error' && seriesMessage.textContent.startsWith('Loaded ')) {
        seriesMessage.hidden = true;
      }
      return;
    }
    label.hidden = false;
    if (seriesMessage.dataset.state !== 'error') seriesMessage.hidden = false;
    bar.max = Math.max(1, progress.total);
    bar.value = progress.scraped;
    bar.hidden = !progress.active;
    const saved = `${progress.scraped.toLocaleString()} / ${progress.total.toLocaleString()} posters saved`;
    const pass = progress.active ? ` · ${progress.completed.toLocaleString()} checked this pass` : '';
    const failures = progress.failed ? ` · ${progress.failed.toLocaleString()} unavailable` : '';
    label.textContent = progress.active
      ? `Saving Sonarr metadata and artwork to encrypted shared metadata: ${saved}${pass}${failures}`
      : progress.scraped >= progress.total
        ? `Sonarr artwork cache complete${failures}`
        : `Sonarr artwork cache status: ${saved}${failures}`;
  } catch { /* The status will refresh on the next poll. */ }
}

window.setInterval(() => {
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
  void Promise.all([arrDirectRequest('/radarr/movies'), arrDirectRequest('/settings/radarr')]).then(async ([liveMovies, settings]) => {
    const slugByTmdbId = new Map(liveMovies.map(movie => [Number(movie.tmdbId), movie.titleSlug]));
    for (const movie of movies) movie.titleSlug = slugByTmdbId.get(Number(movie.tmdbId)) || null;
    await probeArrBase('radarr', settings.url || '');
    for (const row of moviesPanel.querySelectorAll('tr')) {
      const id = Number(row.children[1]?.textContent);
      const movie = movies.find(candidate => Number(candidate.tmdbId) === id);
      if (movie) appendArrDetailLink(row.children[0], 'radarr', movie);
    }
  }).catch(() => {});
  for (const movie of movies) {
    const row = document.createElement('tr');
    const titleCell = document.createElement('td'); titleCell.className = 'title-cell';
    const image = document.createElement('img'); image.className = 'poster'; image.alt = ''; image.loading = 'lazy';
    if (movie.loadArtwork) image.dataset.loadArtwork = 'true';
    titleCell.append(image);
    const title = document.createElement('span'); title.textContent = `${movie.title}${movie.year ? ` (${movie.year})` : ''}`; titleCell.append(title);
    const id = document.createElement('td'); id.textContent = movie.tmdbId ? String(movie.tmdbId) : '—';
    const year = document.createElement('td'); year.textContent = movie.year ? String(movie.year) : '—';
    const file = document.createElement('td'); file.innerHTML = `<span class="badge">${movie.hasFile ? 'Present' : 'Missing'}</span>`;
    const monitored = document.createElement('td'); monitored.textContent = movie.monitored ? 'Yes' : 'No';
    const quality = document.createElement('td'); quality.textContent = movie.qualityProfile;
    const size = document.createElement('td'); size.textContent = movie.hasFile ? formatBytes(movie.sizeOnDisk) : '—';
    row.append(titleCell, id, year, file, monitored, quality, size); moviesPanel.append(row);
  }
  const images = [...moviesPanel.querySelectorAll('img[data-load-artwork]')];
  if ('IntersectionObserver' in window) {
    const observer = new IntersectionObserver(entries => {
      for (const entry of entries) if (entry.isIntersecting) {
        observer.unobserve(entry.target);
        const movie = movies[images.indexOf(entry.target)];
        if (movie?.loadArtwork) void movie.loadArtwork().then(url => { entry.target.src = url; }).catch(() => {});
      }
    }, { rootMargin: '200px' });
    images.forEach(image => observer.observe(image));
  } else {
    for (const image of images) {
      const movie = movies[images.indexOf(image)];
      if (movie?.loadArtwork) void movie.loadArtwork().then(url => { image.src = url; }).catch(() => {});
    }
  }
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
  ageIdentityInMemory = identity;
  if (rememberAgeId.checked) localStorage.setItem(ageSessionKey, identity);
  else localStorage.removeItem(ageSessionKey);
  ageInput.value = '';
  updateAgeIdControls();
  if (location.hash === '#movies') void loadArrSettings('radarr');
  if (location.hash === '#tv' || location.hash === '#series') void loadArrSettings('sonarr');
  contentStatus.textContent = 'Fetching ciphertext and decrypting in this browser…';
  try {
    await import('/assets/metadata-export.js');
    const route = await chooseNodeRoute(node, true);
    let movies;
    try {
      movies = await window.rawrarrMiniExport.loadRadarrLibrary(identity, route.base, message => { contentStatus.textContent = message; });
    } catch (error) {
      if (!route.direct || selectedNode !== nodeId) throw error;
      selectedRoute = { nodeId, base: `/nodes/${encodeURIComponent(nodeId)}`, direct: false };
      contentStatus.textContent = 'Direct request failed; retrying through roarlay…';
      movies = await window.rawrarrMiniExport.loadRadarrLibrary(identity, selectedRoute.base, message => { contentStatus.textContent = message; });
    }
    if (selectedNode !== nodeId) return;
    renderMovies(movies);
    contentStatus.textContent = `${movies.length} encrypted Radarr library entries decrypted locally.`;
  } catch (error) {
    if (selectedNode !== nodeId) return;
    moviesPanel.replaceChildren();
    updateAgeIdControls();
    contentStatus.textContent = error.message || 'Could not decrypt metadata in this browser.';
  }
}

document.getElementById('unlock').addEventListener('click', () => { void unlockMetadata(); });
document.getElementById('refresh-movies').addEventListener('click', async event => {
  const button = event.currentTarget;
  button.disabled = true;
  const nodeId = selectedNodeDetails?.nodeId || 'the selected metadata node';
  contentStatus.textContent = `Requesting a movie metadata rescan from ${nodeId}…`;
  try {
    await unlockMetadataVaultForSettings();
    await arrDirectRequest('/metadata/artwork/rescan', { method: 'POST' });
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
      status.textContent = 'No metadata or apex nodes are connected to this account yet.';
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
      nameInput.value = deviceNames[node.nodeId] || node.nodeId;
      nameInput.maxLength = 80;
      nameInput.setAttribute('aria-label', `Name for ${node.nodeId}`);
      nameInput.title = ageIdentity && configNode ? 'Edit the device name; press Enter or leave the field to save.' : 'Unlock the age identity to edit device names.';
      nameInput.disabled = !ageIdentity || !configNode;
      const heading = document.createElement('div');
      heading.className = 'device-heading';
      heading.append(nameInput, state);
      const detail = document.createElement('small');
      detail.className = 'device-detail';
      detail.textContent = `${roleLabel(node.role)}${node.online ? '' : ' · last seen earlier'}`;
      card.append(heading, detail);
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
        if (event.key === 'Escape') { nameInput.value = deviceNames[node.nodeId] || node.nodeId; nameInput.blur(); }
      });
      nameInput.addEventListener('change', async () => {
        const cleaned = nameInput.value.trim();
        if (!cleaned || cleaned.length > 80 || /[\u0000-\u001f\u007f]/.test(cleaned)) {
          nameInput.value = deviceNames[node.nodeId] || node.nodeId;
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
        }
      });
      nodesPanel.append(card);
      if (node.role === 'metadata' && (node.relayAvailable || node.directUrl) && node.online) {
        if (!defaultNode) defaultNode = node;
      }
    }
    renderDeviceTopology(nodes, observedSyncthingPeers, deviceNames);
    status.textContent = 'Choose a connected metadata or apex node to open its Rawrarr page.';
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
      // Apex nodes can be opened explicitly, but they cannot provide the
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
    pendingPanel.replaceChildren();
    if (!pending.length) return;
    const title = document.createElement('strong');
    title.textContent = 'Nodes waiting for approval';
    pendingPanel.append(title);
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
      item.append(label, approve);
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
