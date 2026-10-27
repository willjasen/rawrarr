const configElement = document.getElementById('payment-config');
const donateButton = document.getElementById('donate');
const subscribeButton = document.getElementById('subscribe');
const planSelect = document.getElementById('plan');
let config = null;

async function requestJson(url, options = {}) {
  const response = await fetch(url, { headers: { Accept: 'application/json', ...(options.body ? { 'Content-Type': 'application/json' } : {}) }, ...options });
  const data = await response.json();
  if (!response.ok) throw new Error(data.error || `Request failed (${response.status})`);
  return data;
}

function providerOrThrow() {
  if (!window.ethereum || typeof window.ethereum.request !== 'function') throw new Error('Install an Ethereum wallet extension to continue.');
  return window.ethereum;
}

async function ensureMainnet(provider) {
  const chain = await provider.request({ method: 'eth_chainId' });
  if (String(chain).toLowerCase() !== '0x1') throw new Error('Switch your wallet to Ethereum mainnet, then try again.');
}

function hexWei(value) {
  return `0x${BigInt(value).toString(16)}`;
}

function selectedPrice() {
  return planSelect.value === 'annual' ? config.annualPriceWei : config.monthlyPriceWei;
}

function updateSubscriptionButton() {
  subscribeButton.disabled = !config?.subscriptionsEnabled || !selectedPrice();
  subscribeButton.textContent = selectedPrice() ? `Subscribe · ${selectedPrice()} wei` : 'Subscription unavailable';
}


async function sendTransfer(amount, expectedAddress = null) {
  if (!config?.treasuryAddress) throw new Error('ETH payments are not configured yet.');
  const provider = providerOrThrow();
  await ensureMainnet(provider);
  const accounts = await provider.request({ method: 'eth_requestAccounts' });
  const from = Array.isArray(accounts) ? accounts[0] : null;
  if (!from) throw new Error('Choose an Ethereum wallet account to continue.');
  if (expectedAddress && from.toLowerCase() !== expectedAddress.toLowerCase()) throw new Error('Switch to the ENS wallet linked to this Rawrarr account before subscribing.');
  const txHash = await provider.request({ method: 'eth_sendTransaction', params: [{ from, to: config.treasuryAddress, value: hexWei(amount) }] });
  return txHash;
}

donateButton.addEventListener('click', async () => {
  const message = document.getElementById('donation-message');
  donateButton.disabled = true;
  try {
    const amount = prompt('Donation amount in ETH (any positive amount):');
    if (amount === null) return;
    if (!/^\d+(\.\d{1,18})?$/.test(amount) || BigInt(amount.split('.')[0]) < 0n || Number(amount) <= 0) throw new Error('Enter an amount greater than 0 with up to 18 decimal places.');
    const [whole, fraction = ''] = amount.split('.');
    const wei = BigInt(whole) * 10n ** 18n + BigInt(fraction.padEnd(18, '0'));
    const { recipient, amountWei } = await requestJson('/api/crypto/donation/link', { method: 'POST', body: JSON.stringify({ amountWei: wei.toString() }) });
    const originalRecipient = config.treasuryAddress;
    config.treasuryAddress = recipient;
    let txHash;
    try { txHash = await sendTransfer(amountWei); }
    finally { config.treasuryAddress = originalRecipient; }
    message.textContent = `Donation sent. Transaction: ${txHash}. It does not change your account tier.`;
  } catch (error) { message.textContent = error.message || 'Could not send donation.'; }
  finally { donateButton.disabled = !config?.donationEnabled; }
});

subscribeButton.addEventListener('click', async () => {
  const message = document.getElementById('subscription-message');
  subscribeButton.disabled = true;
  try {
    const price = selectedPrice();
    if (!price) throw new Error('This plan is not configured yet.');
    const linked = await requestJson('/api/auth/eth/link');
    const expectedAddress = linked.wallets?.[0]?.address;
    if (!expectedAddress) throw new Error('Link an ENS wallet from your account settings before subscribing.');
    const txHash = await sendTransfer(price, expectedAddress);
    message.textContent = 'Transaction sent. Waiting for Ethereum finality…';
    const result = await requestJson('/api/crypto/subscription/claim', { method: 'POST', body: JSON.stringify({ txHash, plan: planSelect.value }) });
    message.textContent = result.status === 'paid' ? `Subscription active through ${new Date(result.paidThrough).toLocaleDateString()}. Renew manually when it expires.` : 'Payment was submitted and is still awaiting finality. Return later to verify it.';
  } catch (error) { message.textContent = error.message || 'Could not verify subscription payment.'; }
  finally { updateSubscriptionButton(); }
});

planSelect.addEventListener('change', updateSubscriptionButton);

try {
  config = await requestJson('/api/crypto/config');
  if (config.donationEnabled && config.treasuryAddress) {
    configElement.textContent = `Ethereum mainnet recipient: ${config.treasuryAddress}`;
    donateButton.disabled = false;
  } else {
    configElement.textContent = 'ETH donations are not configured yet.';
  }
  if (!config.subscriptionsEnabled) configElement.textContent += ' Subscriptions are not configured yet.';
  updateSubscriptionButton();
} catch (error) {
  configElement.textContent = error.message || 'Could not load ETH payment options.';
}
