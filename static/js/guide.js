// Guide toolbar: copy link and print. Both need script, so the buttons start hidden and appear here.
// The guided journey (Guided / Full guide modes) loads as a module; without it the full guide shows.
// Until it has built the guided view, the article is held back so it does not flash first (not for
// visitors who chose the full guide); any failure shows the article.
try {
  if (localStorage.getItem('guideboy:mode') !== '"full"') document.documentElement.dataset.gpending = '';
} catch {}
import('/static/js/guide/journey.mjs').catch(() => delete document.documentElement.dataset.gpending);
setTimeout(() => delete document.documentElement.dataset.gpending, 4000);
document.addEventListener('DOMContentLoaded', () => {
  // The toolbar is sticky. Publish its height so section jumps land below it, and mark it once it is pinned.
  const wrap = document.querySelector('.gwrap');
  const sentinel = document.querySelector('.gsentinel');
  if (wrap) {
    const setHeight = () => document.documentElement.style.setProperty('--tb-h', wrap.offsetHeight + 'px');
    setHeight();
    addEventListener('resize', setHeight);
    if (sentinel && 'IntersectionObserver' in window) {
      new IntersectionObserver(([entry]) => wrap.classList.toggle('is-stuck', !entry.isIntersecting)).observe(sentinel);
    }
  }

  const group = document.querySelector('.tb-group');
  if (!group) return;
  group.hidden = false;
  const status = document.querySelector('[data-status]');

  // Old-style copy through a hidden text box, for browsers that block or lack the clipboard API.
  const legacyCopy = (text) => {
    const box = Object.assign(document.createElement('textarea'), { value: text });
    box.setAttribute('readonly', '');
    box.style.position = 'fixed';
    box.style.opacity = '0';
    document.body.appendChild(box);
    box.select();
    const ok = document.execCommand('copy');
    box.remove();
    if (!ok) throw new Error('copy failed');
  };

  const copyText = async (text) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      legacyCopy(text);
    }
  };

  const copy = group.querySelector('[data-copy]');
  const label = copy.querySelector('span');
  const original = label.textContent;
  let timer;
  copy.addEventListener('click', async () => {
    const url = location.href.split('#')[0];
    try {
      await copyText(url);
      label.textContent = copy.dataset.done;
      if (status) status.textContent = copy.dataset.done;
    } catch {
      window.prompt(original, url);
    }
    clearTimeout(timer);
    timer = setTimeout(() => {
      label.textContent = original;
      if (status) status.textContent = '';
    }, 2000);
  });

  group.querySelector('[data-print]').addEventListener('click', () => window.print());
});
