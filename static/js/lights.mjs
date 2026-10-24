// Card lighting: a soft spotlight and lit rim follow the pointer over cards (the look is in site.css,
// "Lights"). Only for a mouse or pen; touch screens and keyboard focus get the static glow.
const CARDS = '.tcard, .task > a, .jchoice';

export function setupLights(root = document) {
  if (root.dataset?.lights) return;
  if (root.dataset) root.dataset.lights = '1';
  root.addEventListener('pointermove', (e) => {
    if (e.pointerType === 'touch') return;
    const card = e.target.closest(CARDS);
    if (!card) return;
    const box = card.getBoundingClientRect();
    card.style.setProperty('--mx', `${e.clientX - box.left}px`);
    card.style.setProperty('--my', `${e.clientY - box.top}px`);
  });
}
