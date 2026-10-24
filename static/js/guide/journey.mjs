// InteractiveJourney: guided mode for a guide, one stage at a time. It is built in the browser from the
// guide's own sections, so the server-rendered article stays the single source of truth (and is what
// visitors without script, search engines and printers get):
//   situation  a "## ... {#situation}" list of links in the guide, shown as choices (a decision tree)
//   check      before you start, eligibility, fee
//   prepare    documents, as a checklist
//   process    every section with numbered steps, as a timeline
//   finish     the official site, then every other section as a card to open
// Ticks on documents and steps stay in this browser and are shared by both modes (GuideModeSwitcher).
import { h, reduceMotion } from '../directory/dom.mjs';
import { load, save } from '../directory/store.mjs';
import { STAGES, fill, remember, stageOf } from './plan.mjs';
import { setupLights } from '../lights.mjs';
import { decorateTimelines } from './steps.mjs';

const MODE_KEY = 'guideboy:mode';
const RECENT_KEY = 'guideboy:recent';
const LABEL = { situation: 'jm_situation', check: 'jm_check', prepare: 'jm_prepare', process: 'jm_process', finish: 'jm_finish' };
/** Longer items than this (in letters) show only their first sentence in guided mode. */
const SHORT = 90;
const BLOCK = /^(UL|OL|P|DIV|TABLE|BLOCKQUOTE)$/;

// Fewer words: a long list item or paragraph keeps its first sentence (or the part before a block such as a
// nested list); the rest goes behind a "More" button. Screenshots stay visible. The full guide is untouched.
function condense(el, T) {
  if (el.textContent.trim().length <= SHORT) return;
  const kids = [...el.childNodes];
  let seen = 0;
  let cut = -1;
  for (let i = 0; i < kids.length && cut < 0; i++) {
    const n = kids[i];
    if (n.nodeType === 1 && n.matches('input.tick, .term-note')) continue;
    // An item written as paragraphs ("loose" list) keeps its first paragraph, itself shortened.
    if (n.nodeType === 1 && n.tagName === 'P' && !seen) {
      condense(n, T);
      seen += n.textContent.length;
      continue;
    }
    if (n.nodeType === 1 && (BLOCK.test(n.tagName) || n.matches('figure'))) {
      cut = i;
      break;
    }
    if (n.nodeType === 3) {
      const end = /[.:;](?=\s|$)/g;
      let m;
      while ((m = end.exec(n.data))) {
        if (seen + m.index >= 12 && n.data.slice(m.index + 1).trim()) {
          kids.splice(i + 1, 0, n.splitText(m.index + 1));
          cut = i + 1;
          break;
        }
        if (seen + m.index >= 12) break;
      }
    }
    seen += n.textContent.trim().length;
  }
  if (cut < 0) return;
  const rest = kids.slice(cut);
  const shots = rest.filter((n) => n.nodeType === 1 && n.matches('figure'));
  const words = rest.filter((n) => !shots.includes(n));
  if (!words.some((n) => n.textContent.trim())) return;
  const more = h(el.tagName === 'P' ? 'span' : 'div', { class: 'jx', hidden: true });
  more.append(...words);
  const btn = h('button', { type: 'button', class: 'jx-btn', 'aria-expanded': 'false', 'data-more': T.jm_more_btn, 'data-less': T.jm_less_btn }, T.jm_more_btn);
  el.append(btn, more, ...shots);
}

// Small line icons for the situation choices, picked by where a choice leads.
const ICONS = {
  check: 'M20 6 9 17l-5-5',
  prepare: 'M8 3h8l4 4v14H4V3h4zM9 12h6M9 16h6',
  process: 'M4 6h16M4 12h10M4 18h7',
  finish: 'M12 9v4M12 17h.01M10.3 3.9 2 18a2 2 0 0 0 1.7 3h16.6a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z',
  after: 'M5 21V4h11l-1.5 4L16 12H5',
  out: 'M7 17 17 7M8 7h9v9',
};
function choiceIcon(kind) {
  const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');
  svg.setAttribute('aria-hidden', 'true');
  const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
  path.setAttribute('d', ICONS[kind] || ICONS.check);
  svg.append(path);
  return h('span', { class: 'jc-ico' }, svg);
}

// ExpandableExplanation: the first use of each glossary term becomes a button that shows its meaning inline.
function explainTerms(root, glossary, label) {
  if (!glossary.length) return;
  const meaning = new Map(glossary);
  const escaped = [...meaning.keys()].sort((a, b) => b.length - a.length).map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'));
  const pattern = new RegExp(`(?<![\\p{L}\\p{N}])(${escaped.join('|')})(?![\\p{L}\\p{N}])`, 'gu');
  const skip = 'a, h1, h2, h3, h4, button, code, summary, .term-note';
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
    acceptNode: (n) => (n.parentElement.closest(skip) ? NodeFilter.FILTER_REJECT : NodeFilter.FILTER_ACCEPT),
  });
  const nodes = [];
  while (walker.nextNode()) nodes.push(walker.currentNode);
  const used = new Set();
  for (let node of nodes) {
    pattern.lastIndex = 0;
    let m;
    while ((m = pattern.exec(node.data))) {
      if (used.has(m[1])) continue;
      used.add(m[1]);
      const rest = node.splitText(m.index);
      const after = rest.splitText(m[1].length);
      const term = h('button', { type: 'button', class: 'term', 'aria-expanded': 'false', title: label }, m[1]);
      const note = h('span', { class: 'term-note', role: 'note', hidden: true }, meaning.get(m[1]));
      rest.replaceWith(term, note);
      node = after;
      pattern.lastIndex = 0;
    }
  }
}

function start() {
  const data = document.getElementById('journey-text');
  const prose = document.querySelector('.prose');
  const root = document.querySelector('.journey');
  const switcher = document.querySelector('.gmode');
  if (!data || !prose || !root || !switcher) return;
  const J = JSON.parse(data.textContent);
  const T = J.t;

  document.addEventListener('click', (e) => {
    const term = e.target.closest('.term');
    if (!term) return;
    const open = term.getAttribute('aria-expanded') !== 'true';
    term.setAttribute('aria-expanded', String(open));
    term.nextElementSibling.hidden = !open;
  });
  document.addEventListener('click', (e) => {
    const btn = e.target.closest('.jx-btn');
    if (!btn) return;
    const open = btn.getAttribute('aria-expanded') !== 'true';
    btn.setAttribute('aria-expanded', String(open));
    btn.textContent = open ? btn.dataset.less : btn.dataset.more;
    btn.nextElementSibling.hidden = !open;
  });
  explainTerms(prose, J.glossary || [], T.jm_term);

  // ---- the article's sections ----
  const intro = [];
  const sections = [];
  for (const node of [...prose.children]) {
    if (node.tagName === 'H2') sections.push({ id: node.id, title: node.textContent.trim(), nodes: [] });
    else (sections.length ? sections[sections.length - 1].nodes : intro).push(node);
  }
  for (const s of sections) s.stage = stageOf(s.id, s.nodes.some((n) => n.tagName === 'OL'));
  const stageOfId = new Map(sections.map((s) => [s.id, s.stage]));

  // ---- DocumentChecklist and step ticks, added to the article itself so both modes share them ----
  const tickKey = `guideboy:ticks:${J.slug}`;
  const ticks = load('localStorage', tickKey, {});
  const kinds = { d: [], s: [] };
  const addTick = (li, key, kind) => {
    const first = li.querySelector(':scope > p') || li;
    const text = [...first.childNodes].filter((n) => !(n.nodeType === 1 && n.matches('ul, ol, figure'))).map((n) => n.textContent).join('').replace(/\s+/g, ' ').trim().slice(0, 80);
    const box = h('input', { type: 'checkbox', class: 'tick', 'data-tick': key, 'aria-label': `${T.jm_tick}: ${text}` });
    box.checked = Boolean(ticks[key]);
    li.classList.add('tickable');
    li.classList.toggle('is-ticked', box.checked);
    li.prepend(box);
    kinds[kind].push(key);
  };
  let d = 0;
  for (const s of sections) {
    for (const list of s.nodes.filter((n) => n.matches('ul, ol'))) {
      const items = [...list.children].filter((li) => li.tagName === 'LI');
      if (s.stage === 'prepare') items.forEach((li) => addTick(li, `d${d++}`, 'd'));
      else if (s.stage === 'process' && list.tagName === 'OL') items.forEach((li, i) => addTick(li, `${s.id}-${i}`, 's'));
    }
  }
  const count = (kind) => kinds[kind].filter((k) => ticks[k]).length;

  // ---- the guided view: stepper, one panel per stage, back and next ----
  const clone = (nodes) =>
    nodes.map((n) => {
      const c = n.cloneNode(true);
      c.removeAttribute?.('id');
      c.querySelectorAll?.('[id]').forEach((x) => x.removeAttribute('id'));
      return c;
    });
  // A stage with one section already says what it is, so its card needs no heading of its own.
  const alone = (s) => sections.filter((x) => x.stage === s.stage).length === 1 && s.stage !== 'situation';
  const card = (s, extra = '') => {
    const box = h('section', { class: `jcard${extra}`, 'data-sec': s.id, 'aria-label': alone(s) ? s.title : false }, alone(s) ? null : h('h3', {}, s.title), clone(s.nodes));
    if (s.stage !== 'situation') {
      for (const el of box.querySelectorAll(':scope > p, :scope > ul > li, :scope > ol > li')) condense(el, T);
    }
    return box;
  };
  const counter = (kind, key) => h('p', { class: 'jcounter', 'data-kind': kind, 'data-label': T[key] });
  const stages = STAGES.filter((st) => st === 'finish' || sections.some((s) => s.stage === st));

  const content = {
    situation: () => sections.filter((s) => s.stage === 'situation').map((s) => {
      const box = card(s, ' jchoices');
      for (const a of box.querySelectorAll('li > a')) {
        a.classList.add('jchoice');
        const href = a.getAttribute('href');
        const target = href.startsWith('#') ? href.slice(1) : '';
        if (target) a.dataset.jump = target;
        else Object.assign(a, { rel: 'noopener noreferrer' });
        a.prepend(choiceIcon(!target ? 'out' : target === 'after' ? 'after' : stageOfId.get(target)));
      }
      return box;
    }),
    check: () => sections.filter((s) => s.stage === 'check').map((s) => card(s)),
    prepare: () => [counter('d', 'jm_ready'), ...sections.filter((s) => s.stage === 'prepare').map((s) => card(s, ' jchecklist')), h('button', { type: 'button', class: 'tb-btn jreset', 'data-reset': 'd' }, T.jm_reset)],
    process: () => [counter('s', 'jm_done'), ...sections.filter((s) => s.stage === 'process').map((s) => card(s, ' jtimeline')), h('button', { type: 'button', class: 'tb-btn jreset', 'data-reset': 's' }, T.jm_reset)],
    finish: () => {
      const more = sections.filter((s) => s.stage === 'finish');
      return [
        h('div', { class: 'jofficial' },
          h('p', {}, T.jm_finish_text),
          h('p', {}, h('a', { class: 'button', href: J.url, rel: 'noopener noreferrer' }, `${T.open_official} ↗`)),
          h('p', { class: 'meta' }, `${T.official_source}: `, h('span', { lang: document.querySelector('.official [lang]')?.lang || false }, J.official)),
          h('p', { class: 'jindep' }, T.jm_independent)),
        more.length ? h('h3', { class: 'jmore-title' }, T.jm_more) : null,
        ...more.map((s) => h('details', { class: 'jmore', 'data-sec': s.id }, h('summary', {}, s.title), h('div', {}, clone(s.nodes)))),
      ];
    },
  };

  const lang = prose.getAttribute('lang');
  const steps = h('ol', { class: 'jsteps' }, stages.map((st, i) => h('li', {}, h('button', { type: 'button', 'data-go': i }, h('span', { class: 'jnum', 'aria-hidden': 'true' }, i + 1), h('span', { class: 'jlabel' }, T[LABEL[st]])))));
  const bar = h('div', { class: 'jbar', 'aria-hidden': 'true' }, h('span'));
  const panels = stages.map((st, i) =>
    h('section', { class: `jstage jstage-${st}`, tabindex: '-1', 'aria-labelledby': `jh-${st}`, hidden: true },
      h('p', { class: 'jcount' }, fill(T.jm_stage, { n: i + 1, m: stages.length })),
      h('h2', { id: `jh-${st}` }, T[LABEL[st]]),
      h('div', { lang: lang || false }, content[st]())));
  const back = h('button', { type: 'button', class: 'tb-btn', 'data-step': '-1' }, `← ${T.jm_back}`);
  const next = h('button', { type: 'button', class: 'tb-btn tb-primary', 'data-step': '1' });
  root.append(steps, bar, ...panels, h('nav', { class: 'jnav' }, back, next));
  decorateTimelines(root, T);
  setupLights(root);

  // ---- JourneyProgress ----
  const stageKey = `guideboy:stage:${J.slug}`;
  let current = Math.min(Math.max(0, load('localStorage', stageKey, 0)), stages.length - 1);
  const refresh = () => {
    for (const p of root.querySelectorAll('.jcounter')) {
      const kind = p.dataset.kind;
      p.textContent = fill(p.dataset.label, { done: count(kind), n: kinds[kind].length });
      p.hidden = !kinds[kind].length;
    }
    for (const b of root.querySelectorAll('[data-reset]')) b.hidden = !count(b.dataset.reset);
    // The first unticked step of each timeline is where the visitor is.
    for (const ol of document.querySelectorAll('.jtimeline > ol')) {
      const items = [...ol.children];
      items.forEach((li) => li.classList.remove('is-current'));
      items.find((li) => !li.classList.contains('is-ticked'))?.classList.add('is-current');
    }
    const total = kinds.d.length + kinds.s.length;
    const done = count('d') + count('s');
    const stageShare = stages.length > 1 ? current / (stages.length - 1) : 1;
    bar.firstChild.style.width = `${Math.round(100 * (total ? (stageShare + done / total) / 2 : stageShare))}%`;
    save('localStorage', RECENT_KEY, remember(load('localStorage', RECENT_KEY, []), { slug: J.slug, done, total, at: Date.now() }));
  };

  const go = (i, target, quiet) => {
    current = Math.min(Math.max(0, i), stages.length - 1);
    panels.forEach((p, n) => (p.hidden = n !== current));
    steps.querySelectorAll('button').forEach((b, n) => {
      if (n === current) b.setAttribute('aria-current', 'step');
      else b.removeAttribute('aria-current');
      b.classList.toggle('is-past', n < current);
    });
    back.hidden = current === 0;
    next.hidden = current === stages.length - 1;
    if (!next.hidden) next.textContent = `${fill(T.jm_next, { stage: T[LABEL[stages[current + 1]]] })} →`;
    save('localStorage', stageKey, current);
    refresh();
    if (quiet) return;
    const panel = panels[current];
    const spot = target && panel.querySelector(`[data-sec="${CSS.escape(target)}"]`);
    if (spot?.tagName === 'DETAILS') spot.open = true;
    const behavior = reduceMotion() ? 'auto' : 'smooth';
    (spot || root).scrollIntoView({ behavior, block: 'start' });
    if (spot) {
      spot.classList.add('is-flash');
      setTimeout(() => spot.classList.remove('is-flash'), 1600);
    }
    panel.focus({ preventScroll: true });
  };

  // ---- events ----
  document.addEventListener('change', (e) => {
    const box = e.target.closest('input[data-tick]');
    if (!box) return;
    const key = box.dataset.tick;
    if (box.checked) ticks[key] = 1;
    else delete ticks[key];
    save('localStorage', tickKey, ticks);
    for (const b of document.querySelectorAll(`input[data-tick="${CSS.escape(key)}"]`)) {
      b.checked = box.checked;
      b.closest('li').classList.toggle('is-ticked', box.checked);
    }
    refresh();
  });
  root.addEventListener('click', (e) => {
    const reset = e.target.closest('[data-reset]');
    if (reset) {
      for (const key of kinds[reset.dataset.reset]) delete ticks[key];
      save('localStorage', tickKey, ticks);
      for (const b of document.querySelectorAll('input[data-tick]')) {
        b.checked = Boolean(ticks[b.dataset.tick]);
        b.closest('li').classList.toggle('is-ticked', b.checked);
      }
      refresh();
      return;
    }
    const stepBtn = e.target.closest('[data-go]');
    if (stepBtn) return go(Number(stepBtn.dataset.go));
    const move = e.target.closest('[data-step]');
    if (move) return go(current + Number(move.dataset.step));
  });
  // Links to a section (the choices, the toolbar's section menu, links in the text) open its stage.
  document.addEventListener('click', (e) => {
    const a = e.target.closest('a[href^="#"]');
    if (!a || root.hidden) return;
    const id = decodeURIComponent(a.getAttribute('href').slice(1));
    const stage = stageOfId.get(id);
    if (!stage) return;
    e.preventDefault();
    a.closest('details.tb-sections')?.removeAttribute('open');
    go(stages.indexOf(stage), id);
  });

  // ---- GuideModeSwitcher ----
  const setMode = (mode, initial) => {
    const guided = mode === 'guided';
    root.hidden = !guided;
    prose.hidden = guided;
    for (const b of switcher.querySelectorAll('[data-mode]')) b.setAttribute('aria-pressed', String(b.dataset.mode === mode));
    document.documentElement.dataset.gmode = mode;
    save('localStorage', MODE_KEY, mode);
    if (guided && !initial) go(current);
  };
  switcher.addEventListener('click', (e) => {
    const b = e.target.closest('[data-mode]');
    if (b) setMode(b.dataset.mode);
  });
  switcher.hidden = false;
  const hashId = decodeURIComponent(location.hash.slice(1));
  setMode(load('localStorage', MODE_KEY, 'guided') === 'full' ? 'full' : 'guided', true);
  if (!root.hidden) {
    if (stageOfId.has(hashId)) go(stages.indexOf(stageOfId.get(hashId)), hashId);
    else go(current, null, true);
  }
  refresh();
}

function boot() {
  try {
    start();
  } finally {
    delete document.documentElement.dataset.gpending;
  }
}

if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', boot);
else boot();
