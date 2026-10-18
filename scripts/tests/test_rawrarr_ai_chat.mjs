import test from 'node:test';
import assert from 'node:assert/strict';
import { answerMovieQuery, readAiStream } from '../../rust-arr/src/coordinator-ai.mjs';

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
