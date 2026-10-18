// The interactive map of India on the guides pages. The server renders every region (links for states
// with a page, plain named shapes for the rest) and a list of the same regions, so the map works without
// script; this adds the tooltip, Space to open, filtering of the list, and ties the map to the state rail.
import { $, $$ } from './dom.mjs';
import { normalize } from './filters.mjs';

export function setupIndiaMap(root) {
  const wrap = $('.imap-wrap', root);
  const tip = $('.imap-tip', root);
  const regions = $$('.imap-r', root);
  // The script's tooltip replaces the browser's slower one.
  regions.forEach((r) => $('title', r)?.remove());

  const place = (x, y) => {
    const box = wrap.getBoundingClientRect();
    tip.style.setProperty('--x', `${x - box.left}px`);
    tip.style.setProperty('--y', `${y - box.top}px`);
  };
  const show = (r, x, y) => {
    regions.forEach((o) => o.classList.toggle('is-hot', o === r));
    tip.textContent = r.dataset.note ? `${r.dataset.name} · ${r.dataset.note}` : r.dataset.name;
    tip.classList.toggle('is-off', Boolean(r.dataset.note));
    tip.hidden = false;
    place(x, y);
  };
  const hide = (r) => {
    r.classList.remove('is-hot');
    tip.hidden = true;
  };
  const centre = (r) => {
    const b = r.getBoundingClientRect();
    return [b.left + b.width / 2, b.top];
  };

  for (const r of regions) {
    r.addEventListener('pointermove', (e) => show(r, e.clientX, e.clientY));
    r.addEventListener('pointerleave', () => hide(r));
    if (r.tagName.toLowerCase() !== 'a') continue;
    r.addEventListener('focus', () => show(r, ...centre(r)));
    r.addEventListener('blur', () => hide(r));
    // Links open with Enter by themselves; Space opens them too, as on a button.
    r.addEventListener('keydown', (e) => {
      if (e.key === ' ') {
        e.preventDefault();
        location.href = r.getAttribute('href');
      }
    });
  }

  // The state rail and the map point at the same pages: hovering a chip lights its state, and on the
  // main page the state in view is marked as the rail's scroll spy moves.
  const chips = $$('.railchip[data-cat]').filter((c) => c.dataset.cat);
  const byId = new Map(regions.map((r) => [r.dataset.id, r]));
  for (const chip of chips) {
    const r = byId.get(chip.dataset.cat);
    if (!r) continue;
    chip.addEventListener('pointerenter', () => r.classList.add('is-hot'));
    chip.addEventListener('pointerleave', () => r.classList.remove('is-hot'));
  }
  const syncSpy = () => {
    const current = chips.find((c) => c.getAttribute('aria-current') === 'true')?.dataset.cat;
    regions.forEach((r) => r.classList.toggle('is-spy', r.dataset.id === current && !r.hasAttribute('aria-current')));
  };
  // States with a character (Tamil Nadu, Kerala) pop it out of the map when hovered.
  import('./state-character.mjs').then((m) => {
    for (const [id, character] of Object.entries(m.CHARACTERS)) {
      const region = byId.get(id);
      if (region && region.tagName.toLowerCase() === 'a') m.setupStateCharacter(wrap, region, character);
    }
  });

  const rail = $('.rail-in');
  if (rail) new MutationObserver(syncSpy).observe(rail, { subtree: true, attributes: true, attributeFilter: ['aria-current'] });

  // The list for small screens: type to narrow it down.
  const input = $('.imap-q', root);
  const items = $$('.imap-list li', root);
  if (input) {
    input.hidden = false;
    input.addEventListener('input', () => {
      const q = normalize(input.value);
      items.forEach((li) => (li.hidden = q !== '' && !normalize(li.dataset.name).includes(q)));
    });
    $('.imap-alt', root).addEventListener('toggle', (e) => e.target.open && input.focus());
  }
}
