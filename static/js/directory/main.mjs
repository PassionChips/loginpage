// Free tools directory controller: reads the tools from the server-rendered cards (the single source of
// truth), keeps one state object, and re-renders the list, filters, tray and dialogs from it.
import { $, $$, h } from './dom.mjs';
import { load, save } from './store.mjs';
import { matches, normalize, sorter, visibleInCategory } from './filters.mjs';
import { createSaved } from './saved.mjs';
import { createCompare, renderTray, renderTable } from './compare.mjs';
import { renderDetails } from './details.mjs';
import { createPalette } from './palette.mjs';
import { setupRail } from './rail.mjs';
import { createToast } from './toast.mjs';

const VIEW_KEY = 'guideboy:view';

function start() {
  const input = $('#lq');
  const count = $('#lcount');
  if (!input || !count) return;
  const T = JSON.parse($('#dx-text').textContent);
  const groups = $$('.lgroup');
  const listsRoot = $('.lgroups');
  const emptyEl = $('.lempty.empty');
  const noSavedEl = $('.dx-nosaved');
  const toast = createToast($('.dx-toast'));

  const tools = $$('.tcard').map((el, i) => ({
    el,
    i,
    url: el.dataset.url,
    name: el.dataset.name,
    desc: el.dataset.desc,
    badge: el.dataset.badge,
    cat: el.dataset.cat,
    catName: el.dataset.catname,
    host: el.dataset.host,
    text: normalize(el.dataset.text),
    stars: Number(el.dataset.stars) || 0,
  }));
  // The server hides the extra cards for visitors without script; from here the script decides.
  tools.forEach((t) => t.el.classList.remove('extra'));
  const preview = listsRoot.dataset.page === 'index';
  const byUrl = new Map(tools.map((t) => [t.url, t]));
  const validUrls = new Set(byUrl.keys());
  const byCat = new Map(groups.map((g) => [g.dataset.cat, tools.filter((t) => t.cat === g.dataset.cat)]));
  const chips = $$('.railchip');
  const categories = chips
    .filter((c) => c.dataset.cat)
    .map((c) => ({ id: c.dataset.cat, name: c.querySelector('.rc-name').textContent, count: c.querySelector('b').textContent, href: c.href }));

  const saved = createSaved(validUrls);
  const compare = createCompare(validUrls);
  const view = { layout: 'grid', density: 'comfortable', ...load('sessionStorage', VIEW_KEY, {}) };
  const state = {
    query: new URLSearchParams(location.search).get('q') || '',
    badges: new Set(),
    savedOnly: false,
    sort: 'curated',
  };
  input.value = state.query;

  // Controls that only work with script were rendered hidden.
  $$('.dx-bar, [data-palette], .tcard [data-act][hidden]').forEach((el) => (el.hidden = false));

  // The category is the page itself (main page or one category page), so it is not a filter here.
  const isFiltered = () => Boolean(state.query.trim() || state.badges.size || state.savedOnly);
  const plural = (n) => (n === 1 ? count.dataset.one : count.dataset.many).replace('{n}', n);

  // ---- list rendering ----
  let renderedSort = 'curated';
  function apply() {
    if (state.sort !== renderedSort) {
      for (const g of groups) {
        const list = $('.lcards', g);
        [...byCat.get(g.dataset.cat)].sort(sorter(state.sort)).forEach((t) => list.appendChild(t.el));
      }
      renderedSort = state.sort;
    }
    let shown = 0;
    const filtered = isFiltered();
    for (const g of groups) {
      const all = [...byCat.get(g.dataset.cat)].sort(sorter(state.sort));
      const hits = all.filter((t) => matches(t, state, saved));
      const visible = new Set(visibleInCategory(hits, { preview, filtered }));
      all.forEach((t) => (t.el.hidden = !visible.has(t)));
      g.hidden = hits.length === 0;
      // The count is every match in the category; "See all" leads to the rest when only a preview shows.
      $('.gcount', g).textContent = hits.length;
      const seeAll = $('.seeall', g);
      if (seeAll) seeAll.hidden = filtered;
      shown += hits.length;
    }
    count.textContent = plural(shown);
    const nothingSaved = state.savedOnly && saved.size === 0;
    noSavedEl.hidden = !nothingSaved;
    emptyEl.hidden = shown > 0 || nothingSaved;
    if (!emptyEl.hidden) renderSuggestions();
    renderActive();
    syncUrl();
  }

  function renderSuggestions() {
    const currentCategory = preview ? '' : groups[0]?.dataset.cat;
    const picks = categories.filter((c) => c.id !== currentCategory).slice(0, 6);
    $('.dx-try-list', emptyEl).replaceChildren(
      ...picks.map((c) => {
        const b = h('button', { type: 'button', class: 'dx-tag' }, c.name);
        b.addEventListener('click', () => {
          state.query = '';
          input.value = '';
          state.badges.clear();
          state.savedOnly = false;
          location.href = c.href;
        });
        return b;
      })
    );
  }

  function renderActive() {
    const tags = [];
    const tag = (label, onRemove) => {
      const b = h('button', { type: 'button', class: 'dx-tag', 'aria-label': `${T.dx_remove}: ${label}` }, label, h('span', { 'aria-hidden': 'true' }, ' ×'));
      b.addEventListener('click', onRemove);
      tags.push(b);
    };
    if (state.query.trim()) tag(`“${state.query.trim()}”`, () => ((input.value = state.query = ''), apply()));
    for (const b of state.badges) tag({ free: T.badge_free, tier: T.badge_tier, oss: T.badge_oss }[b], () => (state.badges.delete(b), syncFilterChips(), apply()));
    if (state.savedOnly) tag(T.dx_saved, () => ((state.savedOnly = false), syncFilterChips(), apply()));
    const box = $('.dx-active');
    box.hidden = tags.length === 0;
    $('.dx-active-list', box).replaceChildren(...tags);
  }

  function syncUrl() {
    const url = new URL(location.href);
    if (state.query.trim()) url.searchParams.set('q', state.query.trim());
    else url.searchParams.delete('q');
    history.replaceState(null, '', url);
  }

  // ---- search ----
  let debounce;
  input.addEventListener('input', () => {
    clearTimeout(debounce);
    debounce = setTimeout(() => {
      state.query = input.value;
      apply();
    }, 120);
  });
  input.closest('form').addEventListener('submit', (e) => {
    e.preventDefault();
    clearTimeout(debounce);
    state.query = input.value;
    apply();
  });

  // ---- filters, sort, view ----
  const filterChips = $$('.dx-chip');
  function syncFilterChips() {
    filterChips.forEach((c) => {
      const on = c.hasAttribute('data-saved-filter') ? state.savedOnly : state.badges.has(c.dataset.badge);
      c.setAttribute('aria-pressed', String(on));
    });
    $('[data-saved-count]').textContent = saved.size;
  }
  filterChips.forEach((c) =>
    c.addEventListener('click', () => {
      if (c.hasAttribute('data-saved-filter')) state.savedOnly = !state.savedOnly;
      else if (state.badges.has(c.dataset.badge)) state.badges.delete(c.dataset.badge);
      else state.badges.add(c.dataset.badge);
      syncFilterChips();
      apply();
    })
  );
  $('#dx-sort').addEventListener('change', (e) => {
    state.sort = e.target.value;
    apply();
  });

  function applyView() {
    listsRoot.classList.toggle('is-list', view.layout === 'list');
    listsRoot.classList.toggle('is-compact', view.density === 'compact');
    $$('[data-layout]').forEach((b) => b.setAttribute('aria-pressed', String(b.dataset.layout === view.layout)));
    $$('[data-density]').forEach((b) => b.setAttribute('aria-pressed', String(b.dataset.density === view.density)));
    save('sessionStorage', VIEW_KEY, view);
  }
  $$('[data-layout]').forEach((b) => b.addEventListener('click', () => ((view.layout = b.dataset.layout), applyView())));
  $$('[data-density]').forEach((b) => b.addEventListener('click', () => ((view.density = b.dataset.density), applyView())));

  function clearAll() {
    state.query = '';
    input.value = '';
    state.badges.clear();
    state.savedOnly = false;
    syncFilterChips();
    apply();
  }
  $$('[data-clear]').forEach((b) => b.addEventListener('click', clearAll));

  // ---- categories ----
  setupRail($('.rail'), { chips, groups, isFiltered, spy: preview });

  // ---- bookmarks and comparison ----
  function syncTool(url) {
    const isSaved = saved.has(url);
    const isCompared = compare.has(url);
    const card = byUrl.get(url)?.el;
    const scope = [card, $('#dx-details')].filter(Boolean);
    for (const root of scope) {
      $$(`[data-act="save"]`, root)
        .filter((b) => root === card || b.dataset.url === url)
        .forEach((b) => {
          b.setAttribute('aria-pressed', String(isSaved));
          const label = b.querySelector('span');
          if (label) label.textContent = isSaved ? T.dx_unsave : T.dx_save;
        });
      $$(`[data-act="compare"]`, root)
        .filter((b) => root === card || b.dataset.url === url)
        .forEach((b) => b.setAttribute('aria-pressed', String(isCompared)));
    }
  }

  function toggleSave(url) {
    const added = saved.toggle(url);
    syncTool(url);
    syncFilterChips();
    toast(added ? T.dx_saved_toast : T.dx_removed_toast);
    if (state.savedOnly) apply();
  }

  const compareTools = () => compare.list().map((u) => byUrl.get(u));
  function refreshCompare() {
    renderTray($('.dx-tray'), compareTools(), T, toggleCompare);
    const dialog = $('#dx-compare');
    if (dialog.open) {
      if (compare.list().length === 0) dialog.close();
      else renderTable($('.dx-table-wrap', dialog), compareTools(), T, toggleCompare);
    }
  }
  function toggleCompare(url) {
    const result = compare.toggle(url);
    if (result === 'full') {
      toast(T.dx_compare_limit);
      return;
    }
    if (result === 'added') toast(T.dx_added_compare);
    syncTool(url);
    refreshCompare();
  }
  $('.dx-tray-clear').addEventListener('click', () => {
    const urls = compare.list();
    compare.clear();
    urls.forEach(syncTool);
    refreshCompare();
  });
  $('.dx-tray-go').addEventListener('click', () => {
    if (compare.list().length < 2) {
      toast(T.dx_compare_need);
      return;
    }
    const dialog = $('#dx-compare');
    renderTable($('.dx-table-wrap', dialog), compareTools(), T, toggleCompare);
    dialog.showModal();
  });

  function openDetails(tool) {
    const dialog = $('#dx-details');
    renderDetails(dialog, tool, T, { saved: saved.has(tool.url), compared: compare.has(tool.url) });
    if (!dialog.open) dialog.showModal();
  }

  // One handler for every save / details / compare button, on cards and in dialogs.
  document.addEventListener('click', (e) => {
    const button = e.target.closest('[data-act]');
    if (!button) return;
    const url = button.dataset.url || button.closest('.tcard')?.dataset.url;
    if (!url || !byUrl.has(url)) return;
    if (button.dataset.act === 'save') toggleSave(url);
    else if (button.dataset.act === 'compare') toggleCompare(url);
    else if (button.dataset.act === 'details') openDetails(byUrl.get(url));
  });

  // Dialogs: close buttons and clicks on the dimmed backdrop. Escape is handled by the browser.
  $$('.dx-dialog').forEach((dialog) => {
    dialog.addEventListener('click', (e) => {
      if (e.target === dialog || e.target.closest('[data-close]')) dialog.close();
    });
  });

  // ---- command palette and shortcuts ----
  const palette = createPalette($('#dx-palette'), {
    tools,
    categories,
    onCategory: (id) => (location.href = categories.find((c) => c.id === id).href),
    onTool: (tool) => openDetails(tool),
  });
  $('[data-palette]').addEventListener('click', () => palette.open());
  document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      if (!$('#dx-palette').open) palette.open();
      return;
    }
    const typing = /^(input|textarea|select)$/i.test(document.activeElement.tagName) || document.querySelector('dialog[open]');
    if (e.key === '/' && !typing) {
      e.preventDefault();
      input.focus();
    } else if (e.key === 'Escape' && document.activeElement === input && input.value) {
      input.value = state.query = '';
      apply();
    }
  });

  // ---- reveal on scroll ----
  if ('IntersectionObserver' in window) {
    const reveal = new IntersectionObserver(
      (entries) =>
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add('in');
            reveal.unobserve(entry.target);
          }
        }),
      { threshold: 0.05 }
    );
    groups.forEach((g) => reveal.observe(g));
  } else groups.forEach((g) => g.classList.add('in'));

  applyView();
  tools.forEach((t) => syncTool(t.url));
  syncFilterChips();
  refreshCompare();
  apply();
}

if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start);
else start();
