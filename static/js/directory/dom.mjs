export const $ = (sel, root = document) => root.querySelector(sel);
export const $$ = (sel, root = document) => [...root.querySelectorAll(sel)];

/** Builds an element. Strings become text nodes, so tool data can never inject markup. */
export function h(tag, attrs = {}, ...children) {
  const el = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) {
    if (value === false || value == null) continue;
    if (key === 'class') el.className = value;
    else el.setAttribute(key, value === true ? '' : String(value));
  }
  for (const child of children.flat()) {
    if (child == null || child === false) continue;
    el.append(child instanceof Node ? child : document.createTextNode(String(child)));
  }
  return el;
}

/** Copies an icon the server already rendered, so icons are defined in one place. */
export function cloneIcon(selector) {
  const svg = document.querySelector(selector);
  return svg ? svg.cloneNode(true) : document.createTextNode('');
}

export const reduceMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
