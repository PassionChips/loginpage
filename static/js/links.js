// Free tools directory boot script. It runs in <head> so the page is marked as script-enabled before it
// renders (no flash of hidden-then-shown content), then loads the directory modules.
// Without script the directory still works: every tool is listed and the category bar is jump links.
document.documentElement.classList.add('js');
import('/static/js/directory/main.mjs').catch(() => {
  document.documentElement.classList.remove('js');
});
