// Unit tests for the guided journey's pure rules and the guides page suggestions. Run with: node --test tests/js
import test from 'node:test';
import assert from 'node:assert/strict';
import { fill, near, remember, stageOf, STAGES, suggest } from '../../static/js/guide/plan.mjs';

test('guide sections land in the right stage', () => {
  assert.equal(stageOf('situation', false), 'situation');
  assert.equal(stageOf('eligibility', false), 'check');
  assert.equal(stageOf('fee', false), 'check');
  assert.equal(stageOf('documents', false), 'prepare');
  assert.equal(stageOf('steps', false), 'process');
  assert.equal(stageOf('pay', true), 'process', 'any section with numbered steps is part of the process');
  assert.equal(stageOf('problems', true), 'finish', 'troubleshooting stays a card even with a numbered list');
  assert.equal(stageOf('s7', false), 'finish', 'unknown sections are never dropped');
  assert.deepEqual(STAGES, ['situation', 'check', 'prepare', 'process', 'finish']);
});

test('placeholders are filled and unknown ones kept', () => {
  assert.equal(fill('{done} of {n} ready', { done: 2, n: 7 }), '2 of 7 ready');
  assert.equal(fill('Step {n} of {m}', { n: 1 }), 'Step 1 of {m}');
});

test('recent guides keep one entry per guide, newest first', () => {
  let list = remember([], { slug: 'a', done: 0 });
  list = remember(list, { slug: 'b', done: 1 });
  list = remember(list, { slug: 'a', done: 3 });
  assert.deepEqual(list.map((e) => `${e.slug}${e.done}`), ['a3', 'b1']);
  assert.equal(remember(null, { slug: 'x' }).length, 1, 'bad stored data is ignored');
  assert.equal(remember([1, 2, 3, 4].map((n) => ({ slug: `s${n}` })), { slug: 'new' }).length, 4);
});

test('one typo is forgiven in longer words only', () => {
  assert.ok(near('passport', 'pasport'));
  assert.ok(near('licence', 'lisence'));
  assert.ok(near('learner', 'lerner'));
  assert.ok(!near('pan', 'pen'), 'short words must match exactly');
  assert.ok(!near('passport', 'pxxsport'));
});

test('suggestions rank title matches first and need every word', () => {
  const items = [
    { i: 0, name: 'TNPSC', text: 'tnpsc group exams jobs', ours: false },
    { i: 1, name: 'How to renew your passport', text: 'renew passport reissue', ours: true },
    { i: 2, name: 'Apply for a new passport', text: 'new passport fresh', ours: true },
    { i: 3, name: 'Learner licence', text: 'learner licence driving sarathi', ours: true },
  ];
  assert.deepEqual(suggest(items, 'passport').map((x) => x.i), [1, 2]);
  assert.deepEqual(suggest(items, 'renew pasport').map((x) => x.i), [1], 'typo tolerated, every word needed');
  assert.deepEqual(suggest(items, 'driving').map((x) => x.i), [3]);
  assert.deepEqual(suggest(items, '  '), []);
});
