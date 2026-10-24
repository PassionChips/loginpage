// Guides page extras: GuideSearch suggestions under the search box, CategoryNavigation tabs over the popular
// tasks, and "Continue where you left off" from this browser's guide progress (written by the guide pages).
// Without script the search still filters the directory and every task card shows.
import { $, $$, h } from './dom.mjs';
import { load } from './store.mjs';
import { fill, suggest } from '../guide/plan.mjs';

const RECENT_KEY = 'guideboy:recent';

export function setupTasks() {
  const section = $('.tasks');
  if (!section) return;
  const lang = document.documentElement.lang;

  // ---- category tabs ----
  const tabs = $('.task-cats', section);
  const cards = $$('.task', section);
  tabs.hidden = false;
  tabs.addEventListener('click', (e) => {
    const tab = e.target.closest('[data-task-cat]');
    if (!tab) return;
    for (const t of $$('[data-task-cat]', tabs)) t.setAttribute('aria-pressed', String(t === tab));
    const cat = tab.dataset.taskCat;
    for (const c of cards) c.hidden = Boolean(cat) && c.dataset.cat !== cat;
  });

  // ---- continue where you left off ----
  const bySlug = new Map(cards.map((c) => [c.dataset.slug, c]));
  const recent = (load('localStorage', RECENT_KEY, []) || []).filter((r) => r && bySlug.has(r.slug));
  if (recent.length) {
    const list = $('.recent-list', section);
    for (const r of recent) {
      const card = bySlug.get(r.slug);
      const title = $('strong', card);
      const share = r.total ? Math.round((100 * r.done) / r.total) : 0;
      const bar = h('span', { class: 'recent-bar', 'aria-hidden': 'true' }, h('span'));
      bar.firstChild.style.width = `${share}%`;
      list.append(
        h('li', {}, h('a', { href: `/${lang}/guides/${r.slug}` },
          h('strong', { lang: title.lang || false }, title.textContent),
          r.total ? h('span', {}, fill(list.dataset.progress, { done: r.done, n: r.total })) : null,
          r.total ? bar : null)),
      );
    }
    $('.recent', section).hidden = false;
  }

  // ---- search suggestions ----
  const input = $('#lq');
  if (!input) return;
  const items = $$('.tcard').map((el, i) => ({
    i,
    el,
    name: el.dataset.name,
    desc: el.dataset.desc,
    url: el.dataset.url,
    text: el.dataset.text,
    ours: el.dataset.url.startsWith('/'),
    place: $('.loc', el)?.textContent.trim() || '',
    icon: $('.av', el),
  }));
  const box = h('ul', { id: 'lq-suggest', class: 'suggest', role: 'listbox', 'aria-label': section.dataset.suggest, hidden: true });
  input.closest('.field').append(box);
  Object.entries({ role: 'combobox', 'aria-autocomplete': 'list', 'aria-controls': 'lq-suggest', 'aria-expanded': 'false' }).forEach(([k, v]) => input.setAttribute(k, v));
  let shown = [];
  let active = -1;
  const close = () => {
    box.hidden = true;
    input.setAttribute('aria-expanded', 'false');
    input.removeAttribute('aria-activedescendant');
    active = -1;
  };
  const mark = (i) => {
    active = i;
    $$('[role=option]', box).forEach((o, n) => o.setAttribute('aria-selected', String(n === i)));
    if (i >= 0) input.setAttribute('aria-activedescendant', `lq-opt-${i}`);
    else input.removeAttribute('aria-activedescendant');
  };
  const render = () => {
    shown = suggest(items, input.value);
    box.replaceChildren(
      ...shown.map((it, n) => {
        const external = !it.ours;
        return h('li', { role: 'option', id: `lq-opt-${n}`, 'aria-selected': 'false' },
          h('a', { href: it.url, tabindex: '-1', target: external ? '_blank' : false, rel: external ? 'nofollow noopener noreferrer' : false },
            it.icon ? it.icon.cloneNode(true) : null,
            h('span', { class: 'sg-text' }, h('strong', {}, it.name), h('span', {}, it.place ? `${it.place} · ${it.desc}` : it.desc))));
      }),
    );
    box.hidden = !shown.length;
    input.setAttribute('aria-expanded', String(!box.hidden));
    active = -1;
  };
  input.addEventListener('input', render);
  input.addEventListener('focus', () => input.value.trim() && render());
  input.addEventListener('keydown', (e) => {
    if (box.hidden) return;
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      // Wraps through "no suggestion selected" (-1), which leaves the typed text alone.
      let n = active + (e.key === 'ArrowDown' ? 1 : -1);
      if (n < -1) n = shown.length - 1;
      if (n >= shown.length) n = -1;
      mark(n);
    } else if (e.key === 'Enter' && active >= 0) {
      e.preventDefault();
      $$('a', box)[active].click();
    } else if (e.key === 'Escape') {
      close();
    }
  });
  // Clicking a suggestion must not blur the input first, or the list would close before the click lands.
  box.addEventListener('pointerdown', (e) => e.preventDefault());
  box.addEventListener('click', () => close());
  input.addEventListener('blur', close);
}
