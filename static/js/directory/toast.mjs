// Short confirmation messages, also announced to screen readers through the live region.
export function createToast(el) {
  let timer;
  return (message) => {
    el.textContent = message;
    el.classList.add('show');
    clearTimeout(timer);
    timer = setTimeout(() => el.classList.remove('show'), 2200);
  };
}
