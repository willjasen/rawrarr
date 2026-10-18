function decodeBase64Url(value) {
  if (typeof value !== 'string') {
    throw new Error('The server returned incomplete passkey options. Reload the page and try again.');
  }
  const padded = value.replace(/-/g, '+').replace(/_/g, '/') + '='.repeat((4 - value.length % 4) % 4);
  return Uint8Array.from(atob(padded), character => character.charCodeAt(0));
}

function encodeBase64Url(buffer) {
  return btoa(String.fromCharCode(...new Uint8Array(buffer)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/g, '');
}

export function encodeCredential(credential) {
  const response = credential.response;
  const result = {
    id: credential.id,
    rawId: encodeBase64Url(credential.rawId),
    type: credential.type,
    response: { clientDataJSON: encodeBase64Url(response.clientDataJSON) },
    clientExtensionResults: credential.getClientExtensionResults(),
  };
  if (response.attestationObject) {
    result.response.attestationObject = encodeBase64Url(response.attestationObject);
    if (response.getTransports) result.response.transports = response.getTransports();
  } else {
    result.response.authenticatorData = encodeBase64Url(response.authenticatorData);
    result.response.signature = encodeBase64Url(response.signature);
    result.response.userHandle = response.userHandle ? encodeBase64Url(response.userHandle) : null;
  }
  return result;
}

export function decodeOptions(options) {
  if (!options || typeof options.publicKey !== 'object' || options.publicKey === null) {
    throw new Error('The server returned incomplete passkey options. Reload the page and try again.');
  }
  const decoded = structuredClone(options.publicKey);
  decoded.challenge = decodeBase64Url(decoded.challenge);
  if (decoded.user) decoded.user.id = decodeBase64Url(decoded.user.id);
  for (const item of [...(decoded.allowCredentials || []), ...(decoded.excludeCredentials || [])]) {
    item.id = decodeBase64Url(item.id);
  }
  return decoded;
}
