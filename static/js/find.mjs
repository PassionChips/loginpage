// The front page search: suggestions as you type from every guide and official portal in the guides
// directory, tolerant of typos ("income certifcate") and with Hindi and other keywords (built into each
// entry's words by the server from content/guides/keywords.txt). The visitor's state, once chosen, is
// remembered in this browser and puts that state's entries (and India-wide ones) first.
//
// Without script the form still works: it submits to the guides directory, which runs the same search.
import { h } from './directory/dom.mjs';
import { normalize, terms, wordMatch } from './directory/filters.mjs';

const STATE_KEY = 'guideboy:state';
const MAX = 8;

function start() {
  const form = document.querySelector('.hfind');
  if (!form) return;
  const input = form.querySelector('#hq');
  const select = form.querySelector('#hstate');
  const list = form.querySelector('#hq-list');
  const none = form.querySelector('.hfind-none');
  const T = JSON.parse(document.getElementById('find-text').textContent);
  const lang = document.documentElement.lang;
  const items = JSON.parse(document.getElementById('find-data').textContent).map((it, i) => ({
    ...it,
    i,
    title: normalize(it.n).split(' ').filter(Boolean),
    words: normalize(it.x).split(' ').filter(Boolean),
  }));

  // ---- the visitor's state ----
  let state = '';
  try {
    state = localStorage.getItem(STATE_KEY) || '';
  } catch {}
  if (![...select.options].some((o) => o.value === state)) state = '';
  select.value = state;
  const directory = (q) => `/${lang}/guides${state ? '/' + state : ''}${q ? '?q=' + encodeURIComponent(q) : ''}`;
  // Popular applications that are searches (certificates, ration card) narrow to the chosen state.
  const syncLinks = () => document.querySelectorAll('.papps-grid a[data-q]').forEach((a) => (a.href = directory(a.dataset.q.replace(/\+/g, ' '))));
  select.addEventListener('change', () => {
    state = select.value;
    try {
      if (state) localStorage.setItem(STATE_KEY, state);
      else localStorage.removeItem(STATE_KEY);
    } catch {}
    syncLinks();
    if (input.value.trim()) render();
  });
  syncLinks();

  // ---- ranking: every word must match; title words count most, then exact word starts, then typos ----
  const score = (it, ts) => {
    let total = 0;
    for (const t of ts) {
      if (it.title.some((w) => w.startsWith(t))) total += 3;
      else if (it.words.some((w) => w.startsWith(t))) total += 2;
      else if (wordMatch(it.words, t)) total += 1;
      else return 0;
    }
    return total + (it.g ? 0.5 : 0); // this site's own guides first among equals
  };
  const local = (it) => !state || it.s === state || it.s === 'india';

  let results = [];
  let active = -1;
  const setActive = (i) => {
    active = i;
    [...list.children].forEach((li, j) => li.setAttribute('aria-selected', String(j === i)));
    if (i >= 0) {
      input.setAttribute('aria-activedescendant', `hq-${i}`);
      list.children[i].scrollIntoView({ block: 'nearest' });
    } else input.removeAttribute('aria-activedescendant');
  };
  const close = () => {
    list.hidden = true;
    input.setAttribute('aria-expanded', 'false');
    setActive(-1);
  };
  const open = (it) => {
    if (it.g) location.assign(it.u);
    else window.open(it.u, '_blank', 'noopener,noreferrer'); // official portals open in a new tab, as everywhere else
  };

  function render() {
    const ts = terms(input.value);
    if (!ts.length) {
      close();
      none.hidden = true;
      return;
    }
    const hits = items
      .map((it) => ({ it, s: score(it, ts) }))
      .filter((r) => r.s > 0)
      .sort((a, b) => local(b.it) - local(a.it) || b.s - a.s || a.it.i - b.it.i);
    results = hits.slice(0, MAX).map((r) => r.it);
    list.replaceChildren(
      ...results.map((it, i) => {
        const firstElsewhere = state && !local(it) && (i === 0 || local(results[i - 1]));
        const li = h(
          'li',
          { role: 'option', id: `hq-${i}`, class: `hfind-opt${it.g ? ' is-guide' : ''}`, 'aria-selected': 'false', 'data-elsewhere': firstElsewhere ? T.other : null },
          h('strong', {}, it.n),
          h('span', { class: 'hfind-meta' }, `${it.sn} · ${it.g ? T.guide : new URL(it.u).hostname.replace(/^www\./, '')}`)
        );
        li.addEventListener('mousedown', (e) => {
          e.preventDefault(); // keep focus in the box until the choice is made
          open(it);
        });
        li.addEventListener('mousemove', () => active !== i && setActive(i));
        return li;
      })
    );
    list.hidden = results.length === 0;
    input.setAttribute('aria-expanded', String(!list.hidden));
    setActive(-1);
    none.hidden = results.length > 0;
    none.textContent = results.length ? '' : T.none;
  }

  let debounce;
  input.addEventListener('input', () => {
    clearTimeout(debounce);
    debounce = setTimeout(render, 80);
  });
  input.addEventListener('focus', () => input.value.trim() && render());
  input.addEventListener('blur', () => setTimeout(close, 120));
  input.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowDown' && results.length) {
      e.preventDefault();
      if (list.hidden) render();
      setActive((active + 1) % results.length);
    } else if (e.key === 'ArrowUp' && results.length) {
      e.preventDefault();
      setActive(active <= 0 ? results.length - 1 : active - 1);
    } else if (e.key === 'Enter' && active >= 0) {
      e.preventDefault();
      open(results[active]);
    } else if (e.key === 'Escape') {
      if (!list.hidden) close();
      else input.value = '';
    }
  });
  // Submitting searches the guides directory, in the chosen state when there is one.
  form.addEventListener('submit', (e) => {
    e.preventDefault();
    location.assign(directory(input.value.trim()));
  });

  // The header's search icon and the "/" key both lead here.
  const focusIfAsked = () => location.hash === '#find' && input.focus();
  focusIfAsked();
  addEventListener('hashchange', focusIfAsked);
  document.addEventListener('keydown', (e) => {
    const typing = /^(input|textarea|select)$/i.test(document.activeElement.tagName);
    if (e.key === '/' && !typing && !e.metaKey && !e.ctrlKey) {
      e.preventDefault();
      input.focus();
    }
  });
}

if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start);
else start();
