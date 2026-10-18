import assert from 'node:assert/strict';
import test from 'node:test';
import { dinosaurMainColor, dinosaurWingWord } from '../../rust-arr/src/profile-color.mjs';

const seed = (first, fourth = 0, last = 255) => `${first.toString(16).padStart(2, '0')}${'00'.repeat(2)}${fourth.toString(16).padStart(2, '0')}${'00'.repeat(27)}${last.toString(16).padStart(2, '0')}`;

test('profile name color follows the avatar hue and rare golden palette', () => {
  assert.equal(dinosaurMainColor(seed(0)), 'red');
  assert.equal(dinosaurMainColor(seed(16)), 'orange');
  assert.equal(dinosaurMainColor(seed(128)), 'teal');
  assert.equal(dinosaurMainColor(seed(250)), 'red');
  assert.equal(dinosaurMainColor(seed(0, 0, 4)), 'gold');
});

test('profile name wing word follows the avatar wing draw', () => {
  assert.equal(dinosaurWingWord(seed(0, 255, 255)), 'wingless');
  assert.throws(() => dinosaurWingWord('not-a-digest'), /Invalid dinosaur trait seed/);
});
