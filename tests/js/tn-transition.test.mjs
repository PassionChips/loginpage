// Which navigations play the Tamil Nadu transition. Run with: node --test tests/js
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

const context = {};
vm.runInNewContext(readFileSync(new URL('../../static/js/tn-transition.js', import.meta.url), 'utf8'), Object.assign(context, { globalThis: context, URL }));
const { isTamilNaduPage, wantsTransition } = context.guideboyTransition;
const origin = 'https://guideboy.example';

test('only the Tamil Nadu guides page, in any language, is Tamil Nadu', () => {
  for (const path of ['/en/guides/tamil-nadu', '/ta/guides/tamil-nadu', '/hi/guides/tamil-nadu/']) assert.equal(isTamilNaduPage(path), true, path);
  for (const path of ['/en/guides/kerala', '/en/guides/karnataka', '/en/guides', '/en/guides/tamil-nadu-x', '/en/links/tamil-nadu', '/tamil-nadu']) {
    assert.equal(isTamilNaduPage(path), false, path);
  }
});

test('links to Tamil Nadu play it; every other state and page navigates as before', () => {
  assert.equal(wantsTransition('/en/guides/tamil-nadu', '/en/guides', origin), true);
  assert.equal(wantsTransition('/ta/guides/tamil-nadu', '/ta', origin), true);
  assert.equal(wantsTransition(`${origin}/en/guides/tamil-nadu`, '/en/guides/kerala', origin), true, 'absolute same-site link');
  for (const href of ['/en/guides/kerala', '/en/guides/karnataka', '/en/guides/andhra-pradesh', '/en/guides', '/en/guides/passport-new', 'https://www.tnpsc.gov.in', '#documents']) {
    assert.equal(wantsTransition(href, '/en/guides', origin), false, href);
  }
});

test('changing the Tamil Nadu page language or reloading it does not replay it', () => {
  assert.equal(wantsTransition('/hi/guides/tamil-nadu', '/en/guides/tamil-nadu', origin), false);
  assert.equal(wantsTransition('/en/guides/tamil-nadu', '/en/guides/tamil-nadu', origin), false);
});

test('other sites and bad addresses never play it', () => {
  assert.equal(wantsTransition('https://elsewhere.example/en/guides/tamil-nadu', '/en/guides', origin), false);
  assert.equal(wantsTransition('http://[bad', '/en/guides', origin), false);
});
