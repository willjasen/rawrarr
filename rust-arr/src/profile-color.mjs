const COLORS = [
  [15, 'red'],
  [45, 'orange'],
  [75, 'yellow'],
  [105, 'lime'],
  [135, 'green'],
  [165, 'turquoise'],
  [195, 'teal'],
  [225, 'cyan'],
  [255, 'blue'],
  [285, 'indigo'],
  [315, 'purple'],
  [345, 'magenta'],
];

export function dinosaurMainColor(seed) {
  if (!/^[0-9a-f]{64}$/i.test(seed)) throw new Error('Invalid dinosaur color seed');
  const bytes = Uint8Array.from(seed.match(/.{2}/g), byte => Number.parseInt(byte, 16));
  if (bytes[31] < 5) return 'gold';

  const hue = Math.floor(bytes[0] * 360 / 256);
  if (hue >= 345) return 'red';
  return COLORS.find(([limit]) => hue < limit)?.[1] || 'red';
}

export function dinosaurWingWord(seed) {
  if (!/^[0-9a-f]{64}$/i.test(seed)) throw new Error('Invalid dinosaur trait seed');
  const bytes = Uint8Array.from(seed.match(/.{2}/g), byte => Number.parseInt(byte, 16));
  return bytes[3] / 256 < 0.34 ? 'winged' : 'wingless';
}
