// The comparison selection (up to three tools, kept for this browser session), its tray and its table.
import { h, cloneIcon } from './dom.mjs';
import { MAX_COMPARE, toggleLimited } from './filters.mjs';
import { load, save } from './store.mjs';

const KEY = 'guideboy:compare';

export function createCompare(validUrls) {
  let list = load('sessionStorage', KEY, []).filter((u) => validUrls.has(u)).slice(0, MAX_COMPARE);
  const persist = () => save('sessionStorage', KEY, list);
  return {
    has: (url) => list.includes(url),
    list: () => [...list],
    /** Returns 'added', 'removed' or 'full'. */
    toggle(url) {
      const next = toggleLimited(list, url, MAX_COMPARE);
      list = next.list;
      persist();
      return next.result;
    },
    clear() {
      list = [];
      persist();
    },
  };
}

export function renderTray(tray, tools, T, onRemove) {
  tray.hidden = tools.length === 0;
  document.body.classList.toggle('has-tray', tools.length > 0);
  tray.querySelector('.dx-tray-count').textContent = T.dx_selected.replace('{n}', tools.length);
  const list = tray.querySelector('.dx-tray-list');
  list.replaceChildren(
    ...tools.map((tool) => {
      const remove = h('button', { type: 'button', class: 'dx-tag-x', 'aria-label': `${T.dx_remove}: ${tool.name}` }, cloneIcon('.dx-x svg'));
      remove.addEventListener('click', () => onRemove(tool.url));
      return h('li', { lang: 'en' }, h('span', {}, tool.name), remove);
    })
  );
}

const badgePill = (tool, T) =>
  h('span', { class: `badge ${T.show_oss ? `b-${tool.badge}` : 'b-topic'}` }, tool.badgeName);

/** A table with one column per tool and one row per fact the directory actually records. */
export function renderTable(wrap, tools, T, onRemove) {
  const rows = [
    [T.dx_category, (t) => t.catName],
    [T.badge_row, (t) => badgePill(t, T)],
    ...(T.show_oss ? [[T.badge_oss, (t) => (t.badge === 'oss' ? T.dx_yes : T.dx_no)]] : []),
    [T.dx_website, (t) => h('a', { href: t.url, ...(t.url.startsWith('/') ? {} : { target: '_blank', rel: 'nofollow noopener noreferrer' }) }, t.host, ' ', cloneIcon('a.tact .ico'))],
    [T.dx_description, (t) => h('span', { lang: 'en' }, t.desc)],
  ];
  const head = h(
    'tr',
    {},
    h('th', { scope: 'col' }, h('span', { class: 'sr' }, T.dx_category)),
    tools.map((t) => {
      const remove = h('button', { type: 'button', class: 'tb-btn dx-small' }, T.dx_remove);
      remove.addEventListener('click', () => onRemove(t.url));
      return h('th', { scope: 'col' }, h('div', { class: 'dx-th' }, h('strong', { lang: 'en' }, t.name), remove));
    })
  );
  const body = rows.map(([label, cell]) => h('tr', {}, h('th', { scope: 'row' }, label), tools.map((t) => h('td', {}, cell(t)))));
  wrap.replaceChildren(h('table', { class: 'dx-table' }, h('thead', {}, head), h('tbody', {}, body)));
}
