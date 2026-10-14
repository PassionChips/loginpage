// Live search, category filter and reveal-on-scroll for the front page.
// The page is complete without this script; it only makes it faster to use.
document.documentElement.classList.add('js');

document.addEventListener('DOMContentLoaded', () => {
  const input = document.getElementById('q');
  const items = [...document.querySelectorAll('.results li')];
  const chipBox = document.querySelector('.chips');
  const chips = [...document.querySelectorAll('.chip')];
  const count = document.getElementById('count');
  const empty = document.querySelector('.empty');
  if (!input || !count) return;

  let category = '';
  const params = new URLSearchParams(location.search);
  input.value = params.get('q') || '';
  if (chipBox) chipBox.hidden = false;

  const apply = () => {
    const terms = input.value.toLowerCase().split(/\s+/).filter(Boolean);
    let shown = 0;
    items.forEach((li) => {
      const ok = (!category || li.dataset.cat === category) && terms.every((t) => li.dataset.text.includes(t));
      li.hidden = !ok;
      if (ok) {
        li.style.setProperty('--d', Math.min(shown, 8) * 50 + 'ms');
        shown += 1;
      }
    });
    count.textContent = (shown === 1 ? count.dataset.one : count.dataset.many).replace('{n}', shown);
    empty.hidden = shown !== 0 || items.length === 0;

    const url = new URL(location.href);
    input.value.trim() ? url.searchParams.set('q', input.value.trim()) : url.searchParams.delete('q');
    history.replaceState(null, '', url);
  };

  input.addEventListener('input', apply);
  input.closest('form').addEventListener('submit', (e) => {
    e.preventDefault();
    const first = items.find((li) => !li.hidden);
    if (first) first.querySelector('a').click();
  });
  chips.forEach((chip) =>
    chip.addEventListener('click', () => {
      category = chip.dataset.cat;
      chips.forEach((c) => c.setAttribute('aria-pressed', String(c === chip)));
      apply();
    })
  );

  document.addEventListener('keydown', (e) => {
    const typing = /^(input|textarea|select)$/i.test(document.activeElement.tagName);
    if (e.key === '/' && !typing && !e.metaKey && !e.ctrlKey) {
      e.preventDefault();
      input.focus();
    } else if (e.key === 'Escape' && document.activeElement === input) {
      input.value = '';
      apply();
    }
  });
  apply();

  const reveal = [...document.querySelectorAll('.reveal')];
  if (!('IntersectionObserver' in window)) {
    reveal.forEach((el) => el.classList.add('in'));
    return;
  }
  const io = new IntersectionObserver(
    (entries) =>
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          entry.target.classList.add('in');
          io.unobserve(entry.target);
        }
      }),
    { threshold: 0.15 }
  );
  reveal.forEach((el, i) => {
    el.style.setProperty('--d', (i % 3) * 80 + 'ms');
    io.observe(el);
  });
});
