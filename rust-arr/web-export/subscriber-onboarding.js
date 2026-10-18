import { generateHybridIdentity, identityToRecipient } from 'age-encryption';
import { createEncryptedRootBackup, inspectEncryptedRootBackup } from './device-signing-client.js';
import { subscriberCompose } from './subscriber-compose.js';

const byId = id => document.getElementById(id);
let ageIdentity = '';
let rootBackup = '';
let generatedAge = false;
let generatedRoot = false;
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
  byId('deploy-step').hidden = !ageReady || !rootBackup || (generatedRoot && !byId('root-backed-up').checked);
}
function resetRoot() {
  rootBackup = '';
  generatedRoot = false;
  byId('root-fingerprint').textContent = '';
  byId('root-backed-up').checked = false;
  byId('root-backed-up').disabled = true;
  byId('root-backup-label').hidden = true;
  byId('copy-root').hidden = true;
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
}

byId('create-age').addEventListener('click', async event => {
  event.currentTarget.disabled = true;
  try { await setAge(await generateHybridIdentity(), true); byId('age-recipient').textContent += ' · Copy and save the private ID before continuing.'; }
  catch (error) { fail('age-recipient', error); }
  finally { event.currentTarget.disabled = false; }
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
  event.currentTarget.disabled = true;
  try {
    const root = await createEncryptedRootBackup(await identityToRecipient(ageIdentity));
    rootBackup = root.encryptedBackup;
    generatedRoot = true;
    byId('import-root').value = '';
    byId('root-fingerprint').textContent = `Root fingerprint: ${root.fingerprint}`;
    byId('root-backup-label').hidden = false;
    byId('root-backed-up').checked = false;
    byId('root-backed-up').disabled = true;
    byId('copy-root').hidden = false;
    updateSteps();
  } catch (error) { fail('root-fingerprint', error); }
  finally { event.currentTarget.disabled = false; }
});
byId('copy-root').addEventListener('click', async () => {
  try { await navigator.clipboard.writeText(rootBackup); byId('root-backed-up').disabled = false; byId('root-fingerprint').textContent += ' · Encrypted backup copied. Save it outside this browser.'; }
  catch { byId('root-fingerprint').textContent = 'Clipboard access failed. Allow clipboard access and try again.'; }
});
byId('use-root').addEventListener('click', async () => {
  try {
    const backup = byId('import-root').value.trim();
    const root = await inspectEncryptedRootBackup(backup, ageIdentity);
    rootBackup = backup;
    generatedRoot = false;
    byId('import-root').value = '';
    byId('root-fingerprint').textContent = `Verified root fingerprint: ${root.fingerprint}`;
    byId('root-backup-label').hidden = true;
    byId('copy-root').hidden = true;
    updateSteps();
  } catch (error) { fail('root-fingerprint', error); }
});
byId('root-backed-up').addEventListener('change', updateSteps);
byId('node-role').addEventListener('change', () => { byId('media-path-group').hidden = byId('node-role').value !== 'storage'; clearCompose(); });
byId('media-path').addEventListener('input', clearCompose);
byId('prepare-compose').addEventListener('click', () => {
  try {
    const yaml = subscriberCompose({ role: byId('node-role').value, nodeId, coordinatorUrl: location.origin, mediaPath: byId('media-path').value });
    byId('compose-preview').textContent = yaml;
    byId('compose-preview').hidden = false;
    byId('download-compose').hidden = false;
    byId('deploy-instructions').hidden = false;
    byId('compose-message').textContent = 'Review the file and the image prerequisite before running Docker Compose.';
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
});
window.addEventListener('pagehide', () => { ageIdentity = ''; rootBackup = ''; byId('import-age').value = ''; byId('import-root').value = ''; });
