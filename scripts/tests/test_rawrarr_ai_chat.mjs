import test from 'node:test';
import assert from 'node:assert/strict';
import { answerMovieQuery, initAiChat, readAiStream } from '../../rust-arr/src/coordinator-ai.mjs';

const movies = [
  { title: 'Autumn Sky', year: 1994, genres: ['Drama'] },
  { title: 'Moon Harbor', year: 1998, genres: ['Comedy'] },
  { title: 'Winter Road', year: 2002, genres: ['Drama'] },
  { title: 'Unknown Year', year: null, genres: ['Drama'] },
];

test('a 90s question only names matching local movies', () => {
  const answer = answerMovieQuery({ kind: 'movie_filter', startYear: 1990, endYear: 1999, genre: null }, movies);
  assert.match(answer, /I found 2 movies from 1990–1999/);
  assert.match(answer, /Autumn Sky \(1994\)/);
  assert.match(answer, /Moon Harbor \(1998\)/);
  assert.doesNotMatch(answer, /Winter Road|Unknown Year/);
});

test('genre and year filters combine without sending movie data away', () => {
  const answer = answerMovieQuery({ kind: 'movie_filter', startYear: 1990, endYear: 1999, genre: 'drama' }, movies);
  assert.match(answer, /I found 1 movie/);
  assert.match(answer, /Autumn Sky/);
  assert.doesNotMatch(answer, /Moon Harbor/);
});

test('locked metadata and unsupported questions are candid', () => {
  assert.match(answerMovieQuery({ kind: 'movie_filter' }, null), /unlock Movies/);
  assert.match(answerMovieQuery({ kind: 'unsupported' }, movies), /saved that question for review/);
});

test('grounded answers use a gently playful voice while staying precise about records', () => {
  const answer = answerMovieQuery({ kind: 'movie_filter', startYear: 1990, endYear: 1999, genre: null }, movies);
  assert.match(answer, /records loaded here/);
  assert.match(answer, /What else are you curious about\?/);
  assert.doesNotMatch(answer, /I watched|I remember|I feel/);
  assert.match(answerMovieQuery({ kind: 'unsupported' }, movies), /closer look/);
});

test('movie and TV file-size shares use the combined reported bytes', () => {
  const localMovies = [{ sizeOnDisk: 3000 }, { sizeOnDisk: 1000 }];
  const localSeries = [{ sizeOnDisk: 6000 }, { sizeOnDisk: 0 }];
  const answer = answerMovieQuery({ kind: 'library_storage_share' }, localMovies, localSeries);
  assert.match(answer, /Movies use 3\.9 KB \(40\.0%\)/);
  assert.match(answer, /TV uses 5\.9 KB \(60\.0%\)/);
  assert.match(answer, /combined 9\.8 KB/);
});

test('missing TV sizes are explained without guessing a percentage', () => {
  assert.match(answerMovieQuery({ kind: 'library_storage_share' }, movies), /could not load TV sizes/);
  assert.match(answerMovieQuery({ kind: 'library_storage_share' }, [{ sizeOnDisk: 0 }], [{ sizeOnDisk: 0 }]), /could not find reported file sizes/);
});

test('streamed OpenAI text arrives across split frames before the grounded result', async () => {
  const encoder = new TextEncoder();
  const chunks = ['event: delta\ndata: {"text":"I’m cur', 'ious. "}\n\nevent: delta\ndata: {"text":"Let me check."}\n\n', 'event: result\ndata: {"status":"completed","intent":{"kind":"movie_filter"}}\n\n'];
  const response = { body: new ReadableStream({
    start(controller) { for (const chunk of chunks) controller.enqueue(encoder.encode(chunk)); controller.close(); },
  }) };
  let introduction = '';
  const result = await readAiStream(response, delta => { introduction += delta; });
  assert.equal(introduction, 'I’m curious. Let me check.');
  assert.equal(result.intent.kind, 'movie_filter');
});

test('voice recording starts and stops through Talk, transcribes, and submits the spoken question', async () => {
  const elements = new Map();
  const element = id => {
    if (!elements.has(id)) elements.set(id, {
      id, hidden: false, disabled: false, textContent: '', value: '', tabIndex: 0,
      children: [], attributes: {}, handlers: {},
      addEventListener(name, handler) { this.handlers[name] = handler; },
      setAttribute(name, value) { this.attributes[name] = value; },
      append(child) { this.children.push(child); },
      replaceChildren() { this.children = []; },
      reset() {}, focus() {},
    });
    return elements.get(id);
  };
  const ids = ['ai-form', 'ai-question', 'ai-answer', 'ai-voice-answer', 'ai-voice-transcript', 'ai-quota', 'ai-history', 'ai-submit', 'ai-talk', 'ai-chat-panel', 'ai-voice-panel', 'ai-open-movies', 'rawrs-launcher', 'rawrs-close'];
  element('ai-voice-panel').hidden = true;
  const originalDocument = globalThis.document;
  const originalNavigator = Object.getOwnPropertyDescriptor(globalThis, 'navigator');
  const originalWindow = globalThis.window;
  const originalFetch = globalThis.fetch;
  const originalMediaRecorder = globalThis.MediaRecorder;
  let activeRecorder;
  let stoppedTracks = 0;
  const submitted = [];
  class Recorder {
    static isTypeSupported(type) { return type === 'audio/webm'; }
    constructor(stream, options) { this.stream = stream; this.options = options; this.state = 'inactive'; activeRecorder = this; }
    start() { this.state = 'recording'; }
    stop() { this.state = 'inactive'; this.ondataavailable({ data: new Blob(['voice'], { type: this.options.mimeType }) }); this.onstop(); }
  }
  globalThis.document = { getElementById: id => ids.includes(id) ? element(id) : null, createElement: () => ({ textContent: '' }) };
  Object.defineProperty(globalThis, 'navigator', { configurable: true, value: { mediaDevices: { async getUserMedia() { return { getTracks: () => [{ stop() { stoppedTracks += 1; } }] }; } } } });
  globalThis.window = { MediaRecorder: Recorder };
  globalThis.MediaRecorder = Recorder;
  globalThis.fetch = async (url, options = {}) => {
    if (url === '/api/ai/status') return { ok: true, async json() { return { available: true, unlimited: true, resetAt: new Date().toISOString() }; } };
    if (url === '/api/ai/requests' && options.method !== 'POST') return { ok: true, async json() { return []; } };
    if (url === '/api/ai/transcribe') return { ok: true, async json() { return { question: 'Show my comedy movies from the 90s' }; } };
    if (url === '/api/ai/requests') { submitted.push(JSON.parse(options.body)); return { ok: true, async json() { return { status: 'completed', intent: { kind: 'movie_filter', startYear: 1990, endYear: 1999, genre: 'comedy' } }; } }; }
    throw new Error(`Unexpected fetch: ${url}`);
  };
  try {
    initAiChat({ getMovies: () => movies, getSeries: () => [], loadSeries: async () => {}, openMovies() {} });
    const talk = element('ai-talk');
    assert.equal(talk.hidden, false);
    const launcher = element('rawrs-launcher');
    assert.equal(element('ai-voice-panel').hidden, true);
    await launcher.handlers.click();
    assert.equal(element('ai-voice-panel').hidden, false);
    assert.equal(launcher.attributes['aria-expanded'], 'true');
    await element('rawrs-close').handlers.click();
    assert.equal(element('ai-voice-panel').hidden, true);
    assert.equal(launcher.attributes['aria-expanded'], 'false');
    await launcher.handlers.click();
    await talk.handlers.click();
    assert.equal(talk.textContent, 'Stop recording');
    assert.match(element('ai-voice-answer').textContent, /I’m listening/);
    await talk.handlers.click();
    await new Promise(resolve => setImmediate(resolve));
    assert.equal(stoppedTracks, 1);
    assert.equal(submitted.length, 1);
    assert.equal(submitted[0].channel, 'voice');
    assert.equal(submitted[0].question, 'Show my comedy movies from the 90s');
    assert.equal(element('ai-voice-transcript').textContent, 'You asked: Show my comedy movies from the 90s');
    assert.match(element('ai-voice-answer').textContent, /Moon Harbor \(1998\)/);
    assert.equal(talk.disabled, false);
  } finally {
    globalThis.document = originalDocument;
    if (originalNavigator) Object.defineProperty(globalThis, 'navigator', originalNavigator);
    else delete globalThis.navigator;
    globalThis.window = originalWindow;
    globalThis.MediaRecorder = originalMediaRecorder;
    globalThis.fetch = originalFetch;
    void activeRecorder;
  }
});

test('floating Talk panel explains when microphone capture is unavailable', () => {
  const ids = ['ai-form', 'ai-question', 'ai-answer', 'ai-voice-answer', 'ai-voice-transcript', 'ai-quota', 'ai-history', 'ai-submit', 'ai-talk', 'ai-chat-panel', 'ai-voice-panel', 'ai-open-movies', 'rawrs-launcher', 'rawrs-close'];
  const elements = Object.fromEntries(ids.map(id => [id, { hidden: false, disabled: false, textContent: '', handlers: {}, attributes: {}, addEventListener(name, handler) { this.handlers[name] = handler; }, setAttribute(name, value) { this.attributes[name] = value; }, append() {}, replaceChildren() {}, reset() {} }]));
  const originalDocument = globalThis.document;
  const originalNavigator = Object.getOwnPropertyDescriptor(globalThis, 'navigator');
  const originalWindow = globalThis.window;
  elements['ai-voice-panel'].hidden = true;
  globalThis.document = { getElementById: id => elements[id] || null, createElement: () => ({}) };
  Object.defineProperty(globalThis, 'navigator', { configurable: true, value: {} });
  globalThis.window = {};
  try {
    initAiChat({ getMovies: () => movies, getSeries: () => [], loadSeries: async () => {}, openMovies() {} });
    assert.equal(elements['ai-talk'].hidden, false);
    assert.equal(elements['ai-talk'].disabled, true);
    assert.match(elements['ai-voice-answer'].textContent, /unavailable in this browser/);
  } finally {
    globalThis.document = originalDocument;
    if (originalNavigator) Object.defineProperty(globalThis, 'navigator', originalNavigator);
    else delete globalThis.navigator;
    globalThis.window = originalWindow;
  }
});
