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
  const page = (direction) => track.scrollBy({ left: direction * (rtl ? -1 : 1) * track.clientWidth * 0.7, behavior: behavior() });
  prev.addEventListener('click', () => page(-1));
  next.addEventListener('click', () => page(1));
  track.addEventListener('scroll', updateButtons, { passive: true });
  addEventListener('resize', updateButtons);
  updateButtons();

  const setCurrent = (id, reveal = true) => {
    chips.forEach((c) => c.setAttribute('aria-current', String(c.dataset.cat === id)));
    const chip = chips.find((c) => c.dataset.cat === id);
    if (chip && reveal) chip.scrollIntoView({ inline: 'center', block: 'nearest', behavior: behavior() });
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
        target.scrollIntoView({ inline: 'nearest', block: 'nearest', behavior: behavior() });
      }
    });
  });

  // Keep the current page's chip in view.
  const here = chips.find((c) => c.getAttribute('aria-current') === 'true');
  if (here) here.scrollIntoView({ inline: 'center', block: 'nearest' });

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
