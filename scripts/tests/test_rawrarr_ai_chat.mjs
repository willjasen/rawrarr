import test from 'node:test';
import assert from 'node:assert/strict';
import { answerMovieQuery } from '../../rust-arr/src/coordinator-ai.mjs';

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
