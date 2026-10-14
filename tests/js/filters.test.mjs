// Unit tests for the directory's pure filtering rules. Run with: node --test tests/js
import test from 'node:test';
import assert from 'node:assert/strict';
import { matches, normalize, sorter, toggleLimited, paletteResults, terms, visibleInCategory, PREVIEW_PER_CATEGORY } from '../../static/js/directory/filters.mjs';

const tool = (i, name, cat, badge, extra = '') => ({ i, name, cat, badge, url: `https://${i}.example`, text: normalize(`${name} ${extra} ${cat}`) });
const tools = [
  tool(0, 'PDF24 Tools', 'pdf', 'free', 'merge split compress'),
  tool(1, 'Stirling-PDF', 'pdf', 'oss', 'self-hosted toolbox'),
  tool(2, 'Audacity', 'audio', 'oss', 'record and edit audio'),
  tool(3, 'Canva', 'design', 'tier', 'templates posters (official)'),
];
const base = () => ({ query: '', badges: new Set(), savedOnly: false });
const none = new Set();

test('search matches the start of words, across name and description', () => {
  const f = { ...base(), query: 'comp' };
  assert.deepEqual(tools.filter((t) => matches(t, f, none)).map((t) => t.i), [0]);
  assert.equal(matches(tools[2], { ...base(), query: 'dacity' }, none), false, 'no mid-word matches');
  assert.equal(matches(tools[3], { ...base(), query: 'official' }, none), true, 'punctuation is ignored');
  assert.equal(matches(tools[1], { ...base(), query: 'PDF self' }, none), true, 'every word must match');
});

test('filters combine with each other and with search', () => {
  const f = { ...base(), badges: new Set(['oss']) };
  assert.deepEqual(tools.filter((t) => matches(t, f, none)).map((t) => t.i), [1, 2]);
  assert.deepEqual(tools.filter((t) => matches(t, { ...f, query: 'record' }, none)).map((t) => t.i), [2]);
  const both = { ...base(), badges: new Set(['free', 'tier']) };
  assert.deepEqual(tools.filter((t) => matches(t, both, none)).map((t) => t.i), [0, 3], 'badges are OR-ed');
});

test('saved-only shows just bookmarked tools', () => {
  const saved = new Set([tools[3].url]);
  assert.deepEqual(tools.filter((t) => matches(t, { ...base(), savedOnly: true }, saved)).map((t) => t.i), [3]);
});

test('sorting: curated keeps file order, A–Z and Z–A sort by name', () => {
  assert.deepEqual([...tools].sort(sorter('curated')).map((t) => t.i), [0, 1, 2, 3]);
  assert.deepEqual([...tools].sort(sorter('az')).map((t) => t.name), ['Audacity', 'Canva', 'PDF24 Tools', 'Stirling-PDF']);
  assert.deepEqual([...tools].sort(sorter('za')).map((t) => t.name), ['Stirling-PDF', 'PDF24 Tools', 'Canva', 'Audacity']);
});

test('comparison is limited to three', () => {
  let r = toggleLimited([], 'a', 3);
  r = toggleLimited(r.list, 'b', 3);
  r = toggleLimited(r.list, 'c', 3);
  assert.deepEqual(r.list, ['a', 'b', 'c']);
  const full = toggleLimited(r.list, 'd', 3);
  assert.equal(full.result, 'full');
  assert.deepEqual(full.list, ['a', 'b', 'c']);
  assert.deepEqual(toggleLimited(r.list, 'b', 3), { list: ['a', 'c'], result: 'removed' });
});

test('palette lists categories first, then tools whose name starts with the query', () => {
  const cats = [{ id: 'pdf', name: 'PDF tools' }, { id: 'audio', name: 'Audio and music' }];
  assert.equal(paletteResults(tools, cats, '').length, 2, 'empty query shows the categories');
  const r = paletteResults(tools, cats, 'pdf');
  assert.deepEqual(r.map((x) => x.kind), ['category', 'tool', 'tool']);
  assert.equal(r[1].item.name, 'PDF24 Tools');
  assert.deepEqual(terms('  Hello,  World '), ['hello', 'world']);
});

test('stars sort puts the most-starred first and keeps tools without a repository after', () => {
  const withStars = tools.map((t, i) => ({ ...t, stars: [500, 90000, 0, 20000][i] }));
  assert.deepEqual([...withStars].sort(sorter('stars')).map((t) => t.name), ['Stirling-PDF', 'Canva', 'PDF24 Tools', 'Audacity']);
});

test('main page previews the first tools per category until the visitor searches or filters', () => {
  const many = Array.from({ length: 12 }, (_, i) => i);
  assert.equal(PREVIEW_PER_CATEGORY, 8);
  assert.deepEqual(visibleInCategory(many, { preview: true, filtered: false }), many.slice(0, 8));
  assert.equal(visibleInCategory(many, { preview: true, filtered: true }).length, 12, 'search reveals every match');
  assert.equal(visibleInCategory(many, { preview: false, filtered: false }).length, 12, 'category pages list everything');
});
