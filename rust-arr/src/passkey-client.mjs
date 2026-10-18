function decodeBase64Url(value) {
  if (typeof value !== 'string') {
    throw new Error('The server returned incomplete passkey options. Reload the page and try again.');
  }
  const padded = value.replace(/-/g, '+').replace(/_/g, '/') + '='.repeat((4 - value.length % 4) % 4);
  return Uint8Array.from(atob(padded), character => character.charCodeAt(0));
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
