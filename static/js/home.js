// Reveal-on-scroll for the front page. The page is complete without this script.
document.documentElement.classList.add('js');

document.addEventListener('DOMContentLoaded', reveal);

function reveal() {
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
}
