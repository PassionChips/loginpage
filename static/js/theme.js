// Theme: Auto (the system's light or dark), Light, Dark or Ocean (light with a blue accent). It runs in
// <head>, before the page is drawn, so a chosen theme never flashes the other one first; the choice is kept
// in this browser. Without script the site follows the system's setting.
(() => {
  const KEY = 'guideboy:theme';
  const THEMES = ['auto', 'light', 'dark', 'ocean'];
  const root = document.documentElement;
  const apply = (theme) => {
    if (theme === 'auto') delete root.dataset.theme;
    else root.dataset.theme = theme;
  };
  let theme = 'auto';
  try {
    const saved = JSON.parse(localStorage.getItem(KEY));
    if (THEMES.includes(saved)) theme = saved;
  } catch {}
  apply(theme);

  document.addEventListener('DOMContentLoaded', () => {
    const pick = document.querySelector('.themepick');
    const select = pick && pick.querySelector('select');
    if (!select) return;
    select.value = theme;
    pick.hidden = false;
    select.addEventListener('change', () => {
      theme = THEMES.includes(select.value) ? select.value : 'auto';
      apply(theme);
      try {
        localStorage.setItem(KEY, JSON.stringify(theme));
      } catch {}
    });
  });
})();
