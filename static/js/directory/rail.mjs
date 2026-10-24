// The category bar: previous/next buttons when it overflows, arrow-key movement between categories, and
// (on the main page) highlighting the category in view. Each chip is a plain link to its category page.
import { reduceMotion } from './dom.mjs';

export function setupRail(nav, { chips, groups, isFiltered, spy: useSpy }) {
  const track = nav.querySelector('.rail-in');
  const prev = nav.querySelector('.rail-btn.prev');
  const next = nav.querySelector('.rail-btn.next');
  const rtl = document.documentElement.dir === 'rtl';
  const behavior = () => (reduceMotion() ? 'auto' : 'smooth');

  const updateButtons = () => {
    const max = track.scrollWidth - track.clientWidth;
    const pos = Math.abs(track.scrollLeft);
    prev.hidden = pos <= 2;
    next.hidden = max <= 2 || pos >= max - 2;
  };
  // Scrolls only the rail sideways to show a chip (centred, or just into view). scrollIntoView would also
  // scroll the page to the rail, which on the guides page sits below the search and the popular tasks.
  const reveal = (chip, center, smooth) => {
    const t = track.getBoundingClientRect();
    const c = chip.getBoundingClientRect();
    let delta = 0;
    if (center) delta = c.left - t.left - (t.width - c.width) / 2;
    else if (c.left < t.left) delta = c.left - t.left;
    else if (c.right > t.right) delta = c.right - t.right;
    if (Math.abs(delta) > 1) track.scrollBy({ left: delta, behavior: smooth ? behavior() : 'auto' });
  };
  const page = (direction) => track.scrollBy({ left: direction * (rtl ? -1 : 1) * track.clientWidth * 0.7, behavior: behavior() });
  prev.addEventListener('click', () => page(-1));
  next.addEventListener('click', () => page(1));
  track.addEventListener('scroll', updateButtons, { passive: true });
  addEventListener('resize', updateButtons);
  updateButtons();

  const setCurrent = (id, show = true) => {
    chips.forEach((c) => c.setAttribute('aria-current', String(c.dataset.cat === id)));
    const chip = chips.find((c) => c.dataset.cat === id);
    if (chip && show) reveal(chip, true, true);
  };

  chips.forEach((chip, i) => {
    chip.addEventListener('keydown', (e) => {
      const forward = rtl ? 'ArrowLeft' : 'ArrowRight';
      const back = rtl ? 'ArrowRight' : 'ArrowLeft';
      let target = null;
      if (e.key === forward) target = chips[i + 1];
      else if (e.key === back) target = chips[i - 1];
      else if (e.key === 'Home') target = chips[0];
      else if (e.key === 'End') target = chips[chips.length - 1];
      if (target) {
        e.preventDefault();
        target.focus();
        reveal(target, false, true);
      }
    });
  });

  // Keep the current page's chip in view.
  const here = chips.find((c) => c.getAttribute('aria-current') === 'true');
  if (here) reveal(here, true, false);

  if (useSpy && 'IntersectionObserver' in window) {
    const spy = new IntersectionObserver(
      (entries) =>
        entries.forEach((entry) => {
          if (entry.isIntersecting && !isFiltered()) setCurrent(entry.target.dataset.cat);
        }),
      { rootMargin: '-25% 0px -65% 0px' }
    );
    groups.forEach((g) => spy.observe(g));
  }

  return { setCurrent };
}
