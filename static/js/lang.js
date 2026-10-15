// Closes the language and sections menus on an outside click or Escape (both menus work without script),
// and wires the footer's copy-email button.
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

// Footer "copy email" button. It starts hidden and only appears when the clipboard can be used;
// without it the address is still a plain mailto link.
document.addEventListener('DOMContentLoaded', () => {
  if (!navigator.clipboard) return;
  document.querySelectorAll('.copy-btn').forEach((btn) => {
    const label = btn.textContent;
    btn.hidden = false;
    btn.addEventListener('click', async () => {
      try {
        await navigator.clipboard.writeText(btn.dataset.copy);
        btn.textContent = btn.dataset.done;
        setTimeout(() => { btn.textContent = label; }, 1800);
      } catch {
        location.href = 'mailto:' + btn.dataset.copy;
      }
    });
  });
});
