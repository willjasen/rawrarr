import { generateHybridIdentity, identityToRecipient } from 'age-encryption';
import { createEncryptedRootBackup, inspectEncryptedRootBackup } from './device-signing-client.js';
import { subscriberCompose } from './subscriber-compose.js';

const byId = id => document.getElementById(id);
let ageIdentity = '';
let rootBackup = '';
let generatedAge = false;
let rootFingerprint = '';
const nodeId = `device-${[...crypto.getRandomValues(new Uint8Array(12))].map(value => value.toString(16).padStart(2, '0')).join('')}`;

function fail(id, error) { byId(id).textContent = error instanceof Error ? error.message : 'Setup failed. Try again.'; }
function clearCompose() {
  byId('compose-preview').textContent = '';
  byId('compose-preview').hidden = true;
  byId('download-compose').hidden = true;
  byId('deploy-instructions').hidden = true;
  byId('compose-message').textContent = '';
}
function updateSteps() {
  const ageReady = Boolean(ageIdentity) && (!generatedAge || byId('age-backed-up').checked);
  byId('root-step').hidden = !ageReady;
  const setupReady = ageReady && Boolean(rootBackup) && byId('root-backed-up').checked;
  byId('setup-complete-actions').hidden = !setupReady;
  byId('deploy-step').hidden = !setupReady || byId('deploy-step').dataset.opened !== 'true';
}
function resetRoot() {
  rootBackup = '';
  rootFingerprint = '';
  byId('root-fingerprint').textContent = '';
  byId('root-backed-up').checked = false;
  byId('root-backed-up').disabled = true;
  byId('root-backup-label').hidden = true;
  byId('copy-root').hidden = true;
  byId('copy-root-fingerprint').hidden = true;
  clearCompose();
  updateSteps();
}
async function setAge(value, isGenerated) {
  const recipient = await identityToRecipient(value);
  if (!recipient.startsWith('age1pq1')) throw new Error('Use a post-quantum age private ID.');
  ageIdentity = value;
  generatedAge = isGenerated;
  byId('import-age').value = '';
  byId('age-backed-up').checked = !isGenerated;
  byId('age-backed-up').disabled = isGenerated;
  byId('age-backup-label').hidden = !isGenerated;
  byId('copy-age').hidden = !isGenerated;
  byId('age-recipient').textContent = `Public recipient: ${recipient.slice(0, 14)}…${recipient.slice(-8)}`;
  resetRoot();
  return recipient;
}

async function useRememberedAgeIdentity() {
  let rememberedIdentity;
  try { rememberedIdentity = localStorage.getItem('rawrarr.browser.age-id'); }
  catch { return; }
  if (!rememberedIdentity) return;
  try {
    const recipient = await setAge(rememberedIdentity, false);
    byId('age-recipient').textContent = `Using age ID saved in this browser. Public recipient: ${recipient.slice(0, 14)}…${recipient.slice(-8)}`;
  } catch {
    byId('age-recipient').textContent = 'The age ID saved in this browser could not be used. Create or import an age ID to continue.';
  }
}

void useRememberedAgeIdentity();

byId('create-age').addEventListener('click', async event => {
  const button = event.currentTarget;
  button.disabled = true;
  try { await setAge(await generateHybridIdentity(), true); byId('age-recipient').textContent += ' · Copy and save the private ID before continuing.'; }
  catch (error) { fail('age-recipient', error); }
  finally { button.disabled = false; }
});
byId('use-age').addEventListener('click', async () => {
  try { await setAge(byId('import-age').value.trim(), false); }
  catch (error) { fail('age-recipient', error); }
});
byId('copy-age').addEventListener('click', async () => {
  try { await navigator.clipboard.writeText(ageIdentity); byId('age-backed-up').disabled = false; byId('age-recipient').textContent = 'Private age ID copied. Save it outside this browser, then confirm below.'; }
  catch { byId('age-recipient').textContent = 'Clipboard access failed. Allow clipboard access and try again.'; }
});
byId('age-backed-up').addEventListener('change', updateSteps);

byId('create-root').addEventListener('click', async event => {
  const button = event.currentTarget;
  button.disabled = true;
  try {
    const root = await createEncryptedRootBackup(await identityToRecipient(ageIdentity));
    rootBackup = root.encryptedBackup;
    rootFingerprint = root.fingerprint;
    byId('import-root').value = '';
    byId('root-fingerprint').textContent = `Instance root fingerprint: ${root.fingerprint}`;
    byId('copy-root-fingerprint').hidden = false;
    byId('root-backup-label').hidden = false;
    byId('root-backed-up').checked = false;
    byId('root-backed-up').disabled = true;
    byId('copy-root').hidden = false;
    updateSteps();
  } catch (error) { fail('root-fingerprint', error); }
  finally { button.disabled = false; }
});
byId('copy-root').addEventListener('click', async () => {
  try { await navigator.clipboard.writeText(rootBackup); byId('root-backed-up').disabled = false; byId('root-fingerprint').textContent += ' · Encrypted backup copied. Save it outside this browser.'; }
  catch { byId('root-fingerprint').textContent = 'Clipboard access failed. Allow clipboard access and try again.'; }
});
byId('copy-root-fingerprint').addEventListener('click', async () => {
  try { await navigator.clipboard.writeText(rootFingerprint); byId('root-fingerprint').textContent = 'Root fingerprint copied. Save it separately from this browser.'; }
  catch { byId('root-fingerprint').textContent = 'Clipboard access failed. Allow clipboard access and try again.'; }
});
byId('use-root').addEventListener('click', async () => {
  try {
    const backup = byId('import-root').value.trim();
    const root = await inspectEncryptedRootBackup(backup, ageIdentity);
    rootBackup = backup;
    rootFingerprint = root.fingerprint;
    byId('import-root').value = '';
    byId('root-fingerprint').textContent = `Verified root fingerprint: ${root.fingerprint}`;
    byId('copy-root-fingerprint').hidden = false;
    byId('root-backup-label').hidden = false;
    byId('root-backed-up').checked = false;
    byId('root-backed-up').disabled = false;
    byId('copy-root').hidden = true;
    updateSteps();
  } catch (error) { fail('root-fingerprint', error); }
});
byId('root-backed-up').addEventListener('change', updateSteps);
byId('add-node-now').addEventListener('click', () => {
  byId('deploy-step').dataset.opened = 'true';
  updateSteps();
  byId('deploy-step').scrollIntoView({ behavior: 'smooth', block: 'start' });
});
byId('deploy-step').querySelector('[data-reopen="deploy-step"]').addEventListener('click', () => {
  byId('deploy-step').dataset.opened = 'true';
  updateSteps();
});
byId('root-fingerprint-confirm').addEventListener('input', clearCompose);
let automaticDeviceName = 'My fossil node';
byId('node-role').addEventListener('change', () => {
  byId('media-path-group').hidden = byId('node-role').value !== 'storage';
  const nextDefault = byId('node-role').value === 'storage' ? 'My boneyard node' : 'My fossil node';
  if (byId('device-name').value === automaticDeviceName) byId('device-name').value = nextDefault;
  automaticDeviceName = nextDefault;
  clearCompose();
});
byId('media-path').addEventListener('input', clearCompose);
byId('device-name').addEventListener('input', clearCompose);
byId('network-mode').addEventListener('change', () => {
  byId('headscale-server-group').hidden = byId('network-mode').value !== 'headscale';
  byId('internal-coordinator-group').hidden = byId('network-mode').value !== 'internal-tailscale';
  clearCompose();
});
byId('headscale-server').addEventListener('input', clearCompose);
byId('internal-coordinator-url').addEventListener('input', clearCompose);
byId('network-auth-key').addEventListener('input', clearCompose);
byId('prepare-compose').addEventListener('click', () => {
  try {
    const authKey = byId('network-auth-key').value.trim();
    if (!authKey) throw new Error('Enter a one-time Tailscale or Headscale auth key.');
    const pinnedFingerprint = byId('root-fingerprint-confirm').value.trim().toLowerCase();
    if (!/^[0-9a-f]{64}$/.test(pinnedFingerprint) || pinnedFingerprint !== rootFingerprint) throw new Error('Re-enter the independently saved root fingerprint exactly.');
    const mode = byId('network-mode').value;
    const coordinatorUrl = mode === 'internal-tailscale' ? byId('internal-coordinator-url').value.trim().replace(/\/$/, '') : location.origin;
    const yaml = subscriberCompose({ role: byId('node-role').value, nodeId, coordinatorUrl, deviceName: byId('device-name').value, mediaPath: byId('media-path').value, networkMode: mode, controlServer: byId('headscale-server').value.trim(), authKey, rootFingerprint: pinnedFingerprint });
    byId('compose-preview').textContent = yaml;
    byId('compose-preview').hidden = false;
    byId('download-compose').hidden = false;
    byId('deploy-instructions').hidden = false;
    const inlineKey = mode === 'internal-tailscale';
    byId('compose-message').textContent = inlineKey
      ? 'The internal Tailscale auth key is included in this Compose preview and download. Keep the file private and remove the key after the node joins.'
      : 'Compose references a local secret file. Save the Headscale auth key separately as tailscale-authkey; the key is not included in this preview or download.';
    byId('auth-key-instruction').textContent = inlineKey
      ? 'The internal Tailscale auth key is embedded in compose.yaml. Restrict the file to your user (for example, chmod 600 compose.yaml), and remove the key from the file after the node joins.'
      : 'Save the Headscale auth key you entered as tailscale-authkey beside compose.yaml, restrict it to your user (for example, chmod 600 tailscale-authkey), and remove it after the node joins.';
  } catch (error) { fail('compose-message', error); }
});
byId('download-compose').addEventListener('click', () => {
  const blob = new Blob([byId('compose-preview').textContent], { type: 'application/yaml' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = 'compose.yaml';
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
  const inlineKey = byId('network-mode').value === 'internal-tailscale';
  byId('compose-message').textContent = inlineKey
    ? 'Compose downloaded with the internal Tailscale auth key embedded. Keep this file private and remove the key from it after the node joins.'
    : 'Compose downloaded. Save the Headscale auth key separately as tailscale-authkey beside compose.yaml; it was not included in the download.';
  byId('network-auth-key').value = '';
  byId('compose-preview').textContent = '';
  byId('compose-preview').hidden = true;
  byId('download-compose').hidden = true;
});
window.addEventListener('pagehide', () => { ageIdentity = ''; rootBackup = ''; rootFingerprint = ''; byId('import-age').value = ''; byId('import-root').value = ''; byId('root-fingerprint-confirm').value = ''; byId('network-auth-key').value = ''; });
