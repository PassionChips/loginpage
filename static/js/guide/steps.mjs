// ProcessTimeline nodes: each numbered step of a guide gets a round node with an icon for what the step asks
// you to do, read from the step's own words ("Pay the fee" -> rupee, "Upload documents" -> upload...), its
// number, and a check mark that draws itself when the step is ticked. The real checkbox stays inside the node,
// so keyboard, screen readers and the shared ticks work as before; the look and motion are in site.css.
import { h } from '../directory/dom.mjs';
import { fill } from './plan.mjs';

/** Step actions, first match wins, with a 24x24 line icon each. Matched on the step's first sentence. */
export const ACTIONS = [
  ['pay', /\b(pay|payment|fee|fees|paid)\b/i, 'M7 4h10M7 8h10M9 4c3.5 0 5.5 1.6 5.5 4S12.5 12 9 12h-1l7 8'],
  ['upload', /\b(upload|scan|photograph|signature)\b/i, 'M12 16V4M7 9l5-5 5 5M5 20h14'],
  ['book', /\b(book|slot|appointment|schedule|date)\b/i, 'M5 6h14v14H5zM5 10h14M9 3v4M15 3v4'],
  ['test', /\b(test|exam|examination|cbt|interview)\b/i, 'M8 3h8l3 3v15H5V3h3M9 12l2 2 4-4M9 17h6'],
  ['print', /\b(print|download|save|pdf|receipt|extract)\b/i, 'M12 4v11M7 10l5 5 5-5M5 20h14'],
  ['submit', /\b(submit|send|forward|file an?|apply)\b/i, 'M21 3 10 14M21 3l-7 18-4-7-7-4z'],
  ['verify', /\b(check|verif\w*|track|status|confirm|review)\b/i, 'M11 4a7 7 0 1 1 0 14 7 7 0 0 1 0-14zM16.5 16.5 21 21'],
  ['visit', /\b(visit|go to|report|attend|office|centre|center|rto|secretariat|college)\b/i, 'M4 21V9l8-5 8 5v12M9 21v-6h6v6'],
  ['fill', /\b(fill|enter|form|details|choose|select|register|sign up)\b/i, 'M4 20h4L19 9l-4-4L4 16zM13 7l4 4'],
  ['open', /\b(open|log ?in|website|portal|visit the|site)\b/i, 'M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18zM3 12h18M12 3c2.5 2.5 3.5 5.5 3.5 9S14.5 18.5 12 21c-2.5-2.5-3.5-5.5-3.5-9S9.5 5.5 12 3z'],
  ['wait', /\b(wait|receive|sms|get|note)\b/i, 'M12 7v5l3 2M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18z'],
];

/**
 * The action of a step from its text: the keyword that comes first in its first sentence (a step usually
 * starts with its verb: "Open Sarathi and choose..." is "open"), else in the whole text.
 */
export function actionOf(text) {
  const first = String(text).split(/[.:](?:\s|$)/)[0];
  for (const src of [first, String(text)]) {
    let best = null;
    for (const [name, re] of ACTIONS) {
      const m = re.exec(src);
      if (m && (best === null || m.index < best.at)) best = { name, at: m.index };
    }
    if (best) return best.name;
  }
  return 'step';
}

const SVG = 'http://www.w3.org/2000/svg';
function svg(cls, d) {
  const el = document.createElementNS(SVG, 'svg');
  el.setAttribute('viewBox', '0 0 24 24');
  el.setAttribute('aria-hidden', 'true');
  el.setAttribute('class', cls);
  const p = document.createElementNS(SVG, 'path');
  p.setAttribute('d', d);
  el.append(p);
  return el;
}

/** Turns each ticked-able step of the timelines under root into an animated node. */
export function decorateTimelines(root, T) {
  for (const ol of root.querySelectorAll('.jtimeline > ol')) {
    const items = [...ol.children].filter((li) => li.tagName === 'LI');
    items.forEach((li, i) => {
      const box = li.querySelector(':scope > input.tick');
      if (!box || li.querySelector(':scope > .jt-node')) return;
      const text = [...li.childNodes].filter((n) => !(n.nodeType === 1 && n.matches('input, ul, ol, figure, .jx, .jx-btn, .term-note'))).map((n) => n.textContent).join(' ');
      const act = actionOf(text);
      const icon = (ACTIONS.find(([name]) => name === act) || [])[2];
      const node = h('label', { class: 'jt-node', 'data-act': act });
      box.replaceWith(node);
      node.append(box, icon ? svg('jt-icon', icon) : h('span', { class: 'jt-icon jt-n', 'aria-hidden': 'true' }, i + 1), svg('jt-check', 'M5 12.5l4.5 4.5L19 7'), h('span', { class: 'jt-num', 'aria-hidden': 'true' }, i + 1));
      li.prepend(h('span', { class: 'jt-step', 'aria-hidden': 'true' }, fill(T.jm_stage, { n: i + 1, m: items.length })));
      li.style.setProperty('--i', String(i));
      li.classList.add('jt-item');
      box.addEventListener('change', () => {
        node.classList.remove('jt-pop');
        void node.offsetWidth; // restart the animation
        node.classList.add('jt-pop');
      });
    });
  }
}
