// Closes the language menu on an outside click or Escape. The menu itself works without script.
document.addEventListener('click', (e) => {
  document.querySelectorAll('.langmenu[open]').forEach((menu) => {
    if (!menu.contains(e.target)) menu.open = false;
  });
});
document.addEventListener('keydown', (e) => {
  if (e.key !== 'Escape') return;
  document.querySelectorAll('.langmenu[open]').forEach((menu) => {
    menu.open = false;
    menu.querySelector('summary').focus();
  });
});
