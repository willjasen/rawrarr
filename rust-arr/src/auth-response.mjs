export async function readJsonResponse(response, fallbackMessage) {
  const body = await response.text();
  let data = null;
  if (body) {
    try {
      data = JSON.parse(body);
    } catch (_) {
      if (response.ok) throw new Error('The account service returned an invalid response. Please try again.');
    }
  }

  if (!response.ok) {
    const statusMessage = response.status === 404
      ? 'This sign-in method is unavailable in this environment.'
      : `${fallbackMessage} (HTTP ${response.status})`;
    throw new Error(data?.error || statusMessage);
  }
  if (data === null) throw new Error('The account service returned an empty response. Please try again.');
  return data;
}
