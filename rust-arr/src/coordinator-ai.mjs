export function answerMovieQuery(intent, movies) {
  if (!Array.isArray(movies)) return 'Could you unlock Movies first? Then I can look through your library.';
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

export function initAiChat({ getMovies, openMovies }) {
  const form = document.getElementById('ai-form');
  if (!form) return { refresh: async () => {} };
  const question = document.getElementById('ai-question');
  const consent = document.getElementById('ai-cloud-consent');
  const answer = document.getElementById('ai-answer');
  const quota = document.getElementById('ai-quota');
  const history = document.getElementById('ai-history');
  const submit = document.getElementById('ai-submit');
  const talk = document.getElementById('ai-talk');
  document.getElementById('ai-open-movies').addEventListener('click', openMovies);
  let recorder = null;

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
        ? `${status.remaining} of ${status.limit} questions left today. Resets ${reset}.`
        : 'OpenAI is not configured on roarlay yet.';
      submit.disabled = !status.available || status.remaining < 1;
      if (talk) talk.disabled = submit.disabled;
      history.replaceChildren();
      for (const record of records) {
        const item = document.createElement('li');
        item.textContent = `${record.question} — ${record.status}`;
        history.append(item);
      }
    } catch (error) { quota.textContent = error.message || 'Could not load the request inbox.'; }
  }

  async function ask(channel) {
    if (!getMovies()) { answer.textContent = 'Unlock Movies first so I can answer from your library.'; return; }
    if (!consent.checked) { answer.textContent = 'Choose whether to send this question to OpenAI.'; return; }
    submit.disabled = true;
    if (talk) talk.disabled = true;
    answer.textContent = 'Interpreting your question…';
    try {
      const response = await fetch('/api/ai/requests', {
        method: 'POST', cache: 'no-store',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id: crypto.randomUUID(), question: question.value.trim(), channel, cloudConsent: true }),
      });
      if (response.status === 429) throw new Error('You have used your five questions today.');
      if (!response.ok) throw new Error('Rawrarr could not interpret this question. The attempt was not counted.');
      const result = await response.json();
      if (result.status === 'pending') answer.textContent = 'This question is still processing. Check the inbox in a moment.';
      else {
        answer.textContent = answerMovieQuery(result.intent, getMovies());
        if (channel === 'voice') speak(answer.textContent);
      }
      if (result.status !== 'pending') form.reset();
    } catch (error) { answer.textContent = error.message || 'Could not ask Rawrarr.'; }
    finally { await refresh(); }
  }

  form.addEventListener('submit', async event => {
    event.preventDefault();
    await ask('text');
  });

  if (talk && navigator.mediaDevices?.getUserMedia && window.MediaRecorder) {
    talk.hidden = false;
    talk.addEventListener('click', async () => {
      if (recorder?.state === 'recording') { recorder.stop(); return; }
      if (!getMovies()) { answer.textContent = 'Unlock Movies first so I can answer from your library.'; return; }
      if (!consent.checked) { answer.textContent = 'Choose whether to send your recording to OpenAI.'; return; }
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
          answer.textContent = 'Transcribing your question…';
          try {
            const audio = new Blob(chunks, { type: mimeType });
            if (audio.size > 2_000_000) throw new Error('That recording is too long. Please try a shorter question.');
            const response = await fetch('/api/ai/transcribe', {
              method: 'POST', cache: 'no-store',
              headers: { 'Content-Type': mimeType, 'X-Rawrarr-Cloud-Consent': 'yes' }, body: audio,
            });
            if (response.status === 429) throw new Error('You have used your five questions today.');
            if (!response.ok) throw new Error('I could not understand that recording. Please try again.');
            question.value = (await response.json()).question;
            await ask('voice');
          } catch (error) { answer.textContent = error.message || 'Voice is unavailable.'; }
          finally { await refresh(); if (recorder === activeRecorder) recorder = null; }
        };
        activeRecorder.start();
        talk.textContent = 'Stop recording';
        answer.textContent = 'I’m listening. Press Stop recording when you’re done.';
        setTimeout(() => { if (activeRecorder.state === 'recording') activeRecorder.stop(); }, 15000);
      } catch (error) {
        stream?.getTracks().forEach(track => track.stop());
        answer.textContent = error.message || 'Microphone access is unavailable.';
      }
    });
  }
  return { refresh };
}
