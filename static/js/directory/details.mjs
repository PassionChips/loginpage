// The tool details dialog, built from the data on the tool's card.
import { h, cloneIcon } from './dom.mjs';

export function renderDetails(dialog, tool, T, state) {
  dialog.querySelector('#dx-details-title').textContent = tool.name;
  dialog.querySelector('#dx-details-title').setAttribute('lang', 'en');
  const avatar = tool.el.querySelector('.av').cloneNode(true);
  const fact = (label, value) => [h('dt', {}, label), h('dd', {}, value)];
  const body = dialog.querySelector('.dx-dbody');
  body.replaceChildren(
    h('div', { class: 'dx-dmeta' }, avatar, h('p', { class: 'meta' }, tool.catName)),
    h('p', { lang: 'en', class: 'dx-ddesc' }, tool.desc),
    h(
      'dl',
      { class: 'dx-facts' },
      fact(T.badge_row, h('span', { class: `badge ${T.show_oss ? `b-${tool.badge}` : 'b-topic'}` }, tool.badgeName)),
      T.show_oss ? fact(T.badge_oss, tool.badge === 'oss' ? T.dx_yes : T.dx_no) : null,
      fact(T.dx_website, tool.host)
    ),
    h(
      'div',
      { class: 'dx-dactions' },
      h('a', { class: 'tb-btn tb-primary', href: tool.url, ...(tool.url.startsWith('/') ? {} : { target: '_blank', rel: 'nofollow noopener noreferrer' }) }, T.dx_visit, ' ', cloneIcon('a.tact .ico')),
      h('button', { type: 'button', class: 'tb-btn', 'data-act': 'save', 'data-url': tool.url, 'aria-pressed': String(state.saved) }, cloneIcon('.tsave svg'), h('span', {}, state.saved ? T.dx_unsave : T.dx_save)),
      h('button', { type: 'button', class: 'tb-btn', 'data-act': 'compare', 'data-url': tool.url, 'aria-pressed': String(state.compared) }, cloneIcon('.tact[data-act="compare"] svg'), h('span', {}, T.dx_compare))
    )
  );
}
