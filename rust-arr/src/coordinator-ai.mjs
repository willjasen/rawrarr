function formatStorageSize(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  const units = ['KB', 'MB', 'GB', 'TB'];
  let value = bytes / 1024;
  for (const unit of units) {
    if (value < 1024 || unit === 'TB') return `${value.toFixed(1)} ${unit}`;
    value /= 1024;
  }
}

export function answerMovieQuery(intent, movies, series = null) {
  if (!Array.isArray(movies)) return 'Could you unlock Movies first? Then I can look through your library.';
  if (intent?.kind === 'library_storage_share') {
    if (!Array.isArray(series)) return 'I can see Movies, but I could not load TV sizes yet. Open TV and check its Sonarr connection, then try again.';
    const totalSize = records => records.reduce((total, item) => {
      const size = Number(item.sizeOnDisk);
      return total + (Number.isFinite(size) && size > 0 ? size : 0);
    }, 0);
    const movieBytes = totalSize(movies);
    const tvBytes = totalSize(series);
    const combined = movieBytes + tvBytes;
    if (!combined) return 'I could not find reported file sizes for Movies or TV yet. What would you like to check next?';
    return `From the file sizes reported by Radarr and Sonarr, Movies use ${formatStorageSize(movieBytes)} (${(movieBytes / combined * 100).toFixed(1)}%) and TV uses ${formatStorageSize(tvBytes)} (${(tvBytes / combined * 100).toFixed(1)}%) of their combined ${formatStorageSize(combined)}. What else are you curious about?`;
  }
  if (intent?.kind !== 'movie_filter') return 'I saved that question for review. I can search your movies by year or genre right now. What would you like to try?';
  const start = Number.isInteger(intent.startYear) ? intent.startYear : null;
  const end = Number.isInteger(intent.endYear) ? intent.endYear : null;
  const genre = typeof intent.genre === 'string' ? intent.genre.trim().toLowerCase() : '';
  const matches = movies.filter(movie => {
    if (start !== null && (!Number.isInteger(movie.year) || movie.year < start)) return false;
    if (end !== null && (!Number.isInteger(movie.year) || movie.year > end)) return false;
    if (genre && !movie.genres?.some(value => value.toLowerCase() === genre)) return false;
    return true;
  });
  const yearLabel = start !== null && end !== null ? ` from ${start}–${end}` : start !== null ? ` from ${start} onward` : end !== null ? ` through ${end}` : '';
  const genreLabel = genre ? ` in ${genre}` : '';
  const header = `I found ${matches.length} movie${matches.length === 1 ? '' : 's'}${yearLabel}${genreLabel} in your unlocked library.`;
  if (!matches.length) return `${header}\nI couldn't find a matching title in the records loaded here.`;
  const titles = matches.slice(0, 30).map(movie => `• ${movie.title}${movie.year ? ` (${movie.year})` : ''}`);
  const more = matches.length > 30 ? `\n…and ${matches.length - 30} more.` : '';
  return `${header}\n${titles.join('\n')}${more}`;
}

export async function readAiStream(response, onDelta) {
  if (!response.body) throw new Error('The response stream is unavailable.');
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = '';
  let result = null;
  const consume = frame => {
    let event = '';
    const data = [];
    for (const line of frame.split('\n')) {
      if (line.startsWith('event:')) event = line.slice(6).trim();
      if (line.startsWith('data:')) data.push(line.slice(5).trimStart());
    }
    if (!data.length) return;
    if (event === 'delta') onDelta(JSON.parse(data.join('\n')).text || '');
    if (event === 'result') result = JSON.parse(data.join('\n'));
  };
  try {
    while (true) {
      const { value, done } = await reader.read();
      buffer = (buffer + decoder.decode(value, { stream: !done })).replaceAll('\r\n', '\n');
      let end;
      while ((end = buffer.indexOf('\n\n')) !== -1) {
        consume(buffer.slice(0, end));
        buffer = buffer.slice(end + 2);
      }
      if (done) break;
    }
  } finally { reader.releaseLock(); }
  if (!result) throw new Error('The answer stream ended before the result arrived.');
  return result;
}

export function initAiChat({ getMovies, getSeries, loadSeries, openMovies }) {
  const form = document.getElementById('ai-form');
  if (!form) return { refresh: async () => {} };
  const question = document.getElementById('ai-question');
  const chatAnswer = document.getElementById('ai-answer');
  const voiceAnswer = document.getElementById('ai-voice-answer');
  const transcript = document.getElementById('ai-voice-transcript');
  const quota = document.getElementById('ai-quota');
  const history = document.getElementById('ai-history');
  const submit = document.getElementById('ai-submit');
  const talk = document.getElementById('ai-talk');
  const chatTab = document.getElementById('ai-tab-chat');
  const voiceTab = document.getElementById('ai-tab-voice');
  const chatPanel = document.getElementById('ai-chat-panel');
  const voicePanel = document.getElementById('ai-voice-panel');
  document.getElementById('ai-open-movies').addEventListener('click', openMovies);
  let recorder = null;

  function selectMode(mode) {
    const chat = mode === 'chat';
    chatTab.setAttribute('aria-selected', String(chat));
    voiceTab.setAttribute('aria-selected', String(!chat));
    chatTab.tabIndex = chat ? 0 : -1;
    voiceTab.tabIndex = chat ? -1 : 0;
    chatPanel.hidden = !chat;
    voicePanel.hidden = chat;
  }
  chatTab.addEventListener('click', () => selectMode('chat'));
  voiceTab.addEventListener('click', () => selectMode('voice'));
  for (const [tab, mode] of [[chatTab, 'chat'], [voiceTab, 'voice']]) {
    tab.addEventListener('keydown', event => {
      if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
      event.preventDefault();
      const next = event.key === 'Home' ? 'chat' : event.key === 'End' ? 'voice' : mode === 'chat' ? 'voice' : 'chat';
      selectMode(next);
      (next === 'chat' ? chatTab : voiceTab).focus();
    });
  }

  function speak(answerText) {
    if (!('speechSynthesis' in window)) return;
    speechSynthesis.cancel();
    const localVoice = speechSynthesis.getVoices().find(voice => voice.localService && voice.lang?.startsWith('en'));
    const spoken = localVoice
      ? answerText.replaceAll('• ', '').replaceAll('\n', '. ')
      : 'I put the answer on your screen. What else are you curious about?';
    const utterance = new SpeechSynthesisUtterance(spoken);
    if (localVoice) utterance.voice = localVoice;
    utterance.rate = 0.95;
    utterance.pitch = 1.08;
    speechSynthesis.speak(utterance);
  }

  async function refresh() {
    try {
      const [statusResponse, listResponse] = await Promise.all([
        fetch('/api/ai/status', { cache: 'no-store' }),
        fetch('/api/ai/requests', { cache: 'no-store' }),
      ]);
      if (!statusResponse.ok || !listResponse.ok) throw new Error('Request inbox is unavailable.');
      const status = await statusResponse.json();
      const records = await listResponse.json();
      const reset = new Date(status.resetAt).toLocaleString();
      quota.textContent = status.available
        ? status.unlimited ? 'Unlimited questions while you test.' : `${status.remaining} of ${status.limit} questions left today. Resets ${reset}.`
        : 'OpenAI is not configured on roarlay yet.';
      submit.disabled = !status.available || (!status.unlimited && status.remaining < 1);
      if (talk) talk.disabled = submit.disabled;
      history.replaceChildren();
      for (const record of records) {
        const item = document.createElement('li');
        item.textContent = `${record.question} — ${record.status}`;
        history.append(item);
      }
    } catch (error) { quota.textContent = error.message || 'Could not load the request inbox.'; }
  }

  async function ask(channel, askedQuestion) {
    const answer = channel === 'voice' ? voiceAnswer : chatAnswer;
    if (!getMovies()) { answer.textContent = 'Unlock Movies first so I can answer from your library.'; return; }
    submit.disabled = true;
    if (talk) talk.disabled = true;
    answer.textContent = 'Interpreting your question…';
    try {
      const response = await fetch('/api/ai/requests', {
        method: 'POST', cache: 'no-store',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: crypto.randomUUID(), question: askedQuestion.trim(), channel, cloudConsent: true, stream: channel === 'text' }),
      });
      if (response.status === 429) throw new Error('You have used your five questions today.');
      if (!response.ok) throw new Error('Rawrarr could not interpret this question. The attempt was not counted.');
      let introduction = '';
      const result = channel === 'text'
        ? await readAiStream(response, delta => {
            introduction += delta;
            answer.textContent = introduction;
          })
        : await response.json();
      if (result.status === 'pending') answer.textContent = 'This question is still processing. Check the inbox in a moment.';
      else {
        if (result.intent?.kind === 'library_storage_share' && !getSeries()) await loadSeries();
        const groundedAnswer = answerMovieQuery(result.intent, getMovies(), getSeries());
        answer.textContent = introduction ? `${introduction}\n\n${groundedAnswer}` : groundedAnswer;
        if (channel === 'voice') speak(answer.textContent);
      }
      if (channel === 'text' && result.status !== 'pending') form.reset();
    } catch (error) { answer.textContent = error.message || 'Could not ask Rawrarr.'; }
    finally { await refresh(); }
  }

  form.addEventListener('submit', async event => {
    event.preventDefault();
    await ask('text', question.value);
  });

  if (talk && navigator.mediaDevices?.getUserMedia && window.MediaRecorder) {
    talk.hidden = false;
    talk.addEventListener('click', async () => {
      if (recorder?.state === 'recording') { recorder.stop(); return; }
      if (!getMovies()) { voiceAnswer.textContent = 'Unlock Movies first so I can answer from your library.'; return; }
      let stream;
      try {
        stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        const mimeType = ['audio/webm', 'audio/mp4', 'audio/ogg'].find(type => MediaRecorder.isTypeSupported(type));
        if (!mimeType) throw new Error('This browser cannot record a supported audio format.');
        const chunks = [];
        const activeRecorder = new MediaRecorder(stream, { mimeType });
        recorder = activeRecorder;
        activeRecorder.ondataavailable = event => { if (event.data.size) chunks.push(event.data); };
        activeRecorder.onstop = async () => {
          stream.getTracks().forEach(track => track.stop());
          talk.textContent = 'Talk';
          talk.disabled = true;
          voiceAnswer.textContent = 'Transcribing your question…';
          try {
            const audio = new Blob(chunks, { type: mimeType });
            if (audio.size > 2_000_000) throw new Error('That recording is too long. Please try a shorter question.');
            const response = await fetch('/api/ai/transcribe', {
              method: 'POST', cache: 'no-store',
              headers: { 'Content-Type': mimeType, 'X-Rawrarr-Cloud-Consent': 'yes' }, body: audio,
            });
            if (response.status === 429) throw new Error('You have used your five questions today.');
            if (!response.ok) throw new Error('I could not understand that recording. Please try again.');
            const spokenQuestion = (await response.json()).question;
            transcript.textContent = `You asked: ${spokenQuestion}`;
            await ask('voice', spokenQuestion);
          } catch (error) { voiceAnswer.textContent = error.message || 'Voice is unavailable.'; }
          finally { await refresh(); if (recorder === activeRecorder) recorder = null; }
        };
        activeRecorder.start();
        talk.textContent = 'Stop recording';
        voiceAnswer.textContent = 'I’m listening. Press Stop recording when you’re done.';
        setTimeout(() => { if (activeRecorder.state === 'recording') activeRecorder.stop(); }, 15000);
      } catch (error) {
        stream?.getTracks().forEach(track => track.stop());
        voiceAnswer.textContent = error.message || 'Microphone access is unavailable.';
      }
    });
  }
  return { refresh };
}
