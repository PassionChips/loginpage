// Command palette (Ctrl/Cmd + K): jump to a category or open a tool's details from the keyboard.
import { h, cloneIcon } from './dom.mjs';
import { paletteResults } from './filters.mjs';

export function createPalette(dialog, { tools, categories, onCategory, onTool }) {
  const input = dialog.querySelector('#dx-pq');
  const list = dialog.querySelector('#dx-plist');
  let results = [];
  let active = 0;

  const highlight = () => {
    [...list.children].forEach((li, i) => li.setAttribute('aria-selected', String(i === active)));
    const current = list.children[active];
    if (current) {
      input.setAttribute('aria-activedescendant', current.id);
      current.scrollIntoView({ block: 'nearest' });
    } else input.removeAttribute('aria-activedescendant');
  };

  const render = () => {
    results = paletteResults(tools, categories, input.value);
    active = 0;
    list.replaceChildren(
      ...results.map((r, i) => {
        const isTool = r.kind === 'tool';
        const li = h(
          'li',
          { role: 'option', id: `dx-opt-${i}`, class: 'dx-opt', 'aria-selected': 'false' },
          isTool ? r.item.el.querySelector('.av').cloneNode(true) : (document.querySelector(`.railchip[data-cat="${r.item.id}"] .stack`)?.cloneNode(true) || h('span', { class: 'dx-opt-ico' })),
          h('span', { class: 'dx-opt-text' }, h('strong', isTool ? { lang: 'en' } : {}, isTool ? r.item.name : r.item.name), h('small', {}, isTool ? r.item.catName : r.item.count))
        );
        li.addEventListener('click', () => pick(i));
        li.addEventListener('mousemove', () => {
          if (active !== i) {
            active = i;
            highlight();
          }
        });
        return li;
      })
    );
    highlight();
  };

  const pick = (i) => {
    const r = results[i];
    if (!r) return;
    dialog.close();
    if (r.kind === 'category') onCategory(r.item.id);
    else onTool(r.item);
  };

  input.addEventListener('input', render);
  input.addEventListener('keydown', (e) => {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      if (!results.length) return;
      active = (active + (e.key === 'ArrowDown' ? 1 : -1) + results.length) % results.length;
      highlight();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      pick(active);
    }
  });

  return {
    open() {
      input.value = '';
      render();
      dialog.showModal();
      input.focus();
    },
  };
}
