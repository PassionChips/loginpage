// Closes the language and sections menus on an outside click or Escape. Both menus work without script.
document.addEventListener('click', (e) => {
  document.querySelectorAll('.langmenu[open], .tb-sections[open]').forEach((menu) => {
    if (!menu.contains(e.target)) menu.open = false;
  });
});
document.addEventListener('keydown', (e) => {
  if (e.key !== 'Escape') return;
  document.querySelectorAll('.langmenu[open], .tb-sections[open]').forEach((menu) => {
    menu.open = false;
    menu.querySelector('summary').focus();
  });
});
