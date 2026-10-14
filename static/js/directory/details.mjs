// The tool details dialog, built from the data on the tool's card.
import { h, cloneIcon } from './dom.mjs';

export function renderDetails(dialog, tool, T, state) {
  dialog.querySelector('#dx-details-title').textContent = tool.name;
  dialog.querySelector('#dx-details-title').setAttribute('lang', 'en');
  const badge = { free: T.badge_free, tier: T.badge_tier, oss: T.badge_oss }[tool.badge];
  const avatar = tool.el.querySelector('.av').cloneNode(true);
  const fact = (label, value) => [h('dt', {}, label), h('dd', {}, value)];
  const body = dialog.querySelector('.dx-dbody');
  body.replaceChildren(
    h('div', { class: 'dx-dmeta' }, avatar, h('p', { class: 'meta' }, tool.catName)),
    h('p', { lang: 'en', class: 'dx-ddesc' }, tool.desc),
    h(
      'dl',
      { class: 'dx-facts' },
      fact(T.dx_pricing, h('span', { class: `badge b-${tool.badge}` }, badge)),
      fact(T.badge_oss, tool.badge === 'oss' ? T.dx_yes : T.dx_no),
      fact(T.dx_website, tool.host)
    ),
    h(
      'div',
      { class: 'dx-dactions' },
      h('a', { class: 'tb-btn tb-primary', href: tool.url, target: '_blank', rel: 'nofollow noopener noreferrer' }, T.dx_visit, ' ', cloneIcon('a.tact .ico')),
      h('button', { type: 'button', class: 'tb-btn', 'data-act': 'save', 'data-url': tool.url, 'aria-pressed': String(state.saved) }, cloneIcon('.tsave svg'), h('span', {}, state.saved ? T.dx_unsave : T.dx_save)),
      h('button', { type: 'button', class: 'tb-btn', 'data-act': 'compare', 'data-url': tool.url, 'aria-pressed': String(state.compared) }, cloneIcon('.tact[data-act="compare"] svg'), h('span', {}, T.dx_compare))
    )
  );
}
