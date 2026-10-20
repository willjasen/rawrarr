// Version this shared module URL so browsers that cached the older passkey
// client cannot keep a registration page stuck before its first status update.
import { decodeOptions, encodeCredential } from '/assets/passkey-client.mjs?v=coordinator-passkey-2';

const message = document.getElementById('message');
const returnLink = document.getElementById('return-link');
const createButton = document.getElementById('create-passkey');
const token = window.location.hash.slice(1);
// Keep the one-time capability out of the visible URL and browser history before use.
history.replaceState(null, '', '/passkeys/add');

let registration = null;

async function post(path, body) {
  const response = await fetch(path, {
    method: 'POST',
    headers: { Accept: 'application/json', 'Content-Type': 'application/json' },
    cache: 'no-store',
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    if (response.status === 410) throw new Error('This link expired or was already used. Start again from your signed-in fossil node.');
    throw new Error('Could not add a passkey. Return to your fossil node and try again.');
  }
  return response.json();
}

async function preparePasskey() {
  if (!/^[0-9a-f]{64}$/i.test(token)) throw new Error('The one-time link is missing or invalid. Start again from your signed-in fossil node.');
  if (!window.PublicKeyCredential || !navigator.credentials) throw new Error('This browser does not support passkeys.');
  registration = await post('/api/passkeys/coordinator/register/start', { token });
  message.textContent = 'Your secure link is ready. Select Create roarlay passkey, then follow your browser’s passkey prompt.';
  createButton.hidden = false;
}

async function addPasskey() {
  if (!registration) return;
  createButton.disabled = true;
  message.textContent = 'Waiting for your browser to create the roarlay passkey…';
  try {
    const credential = await navigator.credentials.create({ publicKey: decodeOptions(registration.options) });
    if (!credential) throw new Error('Passkey registration was cancelled. Select the button to try again.');
    await post('/api/passkeys/coordinator/register/finish', {
      challengeId: registration.challengeId,
      token,
      credential: encodeCredential(credential),
    });
    message.textContent = 'Passkey added to your existing Rawrarr account. Your localhost passkey and current sign-in remain unchanged.';
    createButton.hidden = true;
    returnLink.hidden = false;
  } catch (error) {
    message.textContent = error.message || 'Could not add a passkey. Start again from your signed-in fossil node.';
    createButton.disabled = false;
  }
}

createButton.addEventListener('click', addPasskey);

preparePasskey().catch(error => { message.textContent = error.message || 'Could not prepare passkey registration.'; });
