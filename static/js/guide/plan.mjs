// Pure rules for the guided journey and the guides page. No DOM access, so they are unit-tested in Node.
import { normalize, terms } from '../directory/filters.mjs';

/** The journey's stages, in order. A guide shows only the ones it has content for (finish always). */
export const STAGES = ['situation', 'check', 'prepare', 'process', 'finish'];

const CHECK = new Set(['before', 'eligibility', 'fee']);
const FINISH = new Set(['after', 'help', 'appeal', 'problems', 'good', 'concessions']);

/**
 * Which stage a guide section belongs to, from its id (`## Title {#id}` in the guide) and whether it has a
 * numbered list. Sections with numbered steps become the timeline; anything else unknown is a card to open
 * at the end, so a new guide never loses text in guided mode.
 */
export function stageOf(id, numbered) {
  if (id === 'situation') return 'situation';
  if (CHECK.has(id)) return 'check';
  if (id === 'documents') return 'prepare';
  if (id === 'steps' || (numbered && !FINISH.has(id))) return 'process';
  return 'finish';
}

/** Replaces `{name}` placeholders in an interface string. */
export function fill(text, vars) {
  return String(text).replace(/\{(\w+)\}/g, (all, k) => (k in vars ? String(vars[k]) : all));
}

/** The visitor's recent guides, newest first, one entry per guide. */
export function remember(list, entry, max = 4) {
  return [entry, ...(Array.isArray(list) ? list : []).filter((e) => e && e.slug !== entry.slug)].slice(0, max);
}

/** True when `word` is `term` with at most one letter wrong, missing or extra (for terms of 4+ letters). */
export function near(word, term) {
  if (term.length < 4) return false;
  const w = word.slice(0, term.length + 1);
  for (const cand of [w, w.slice(0, term.length), w.slice(0, term.length - 1)]) {
    if (Math.abs(cand.length - term.length) > 1) continue;
    let i = 0;
    let j = 0;
    let edits = 0;
    while (i < cand.length && j < term.length) {
      if (cand[i] === term[j]) {
        i++;
        j++;
        continue;
      }
      if (++edits > 1) break;
      if (cand.length > term.length) i++;
      else if (cand.length < term.length) j++;
      else {
        i++;
        j++;
      }
    }
    if (edits + (cand.length - i) + (term.length - j) <= 1) return true;
  }
  return false;
}

/**
 * Search suggestions: every word of the query must match a word start (or, with one typo, a word) of the
 * entry. Title matches rank first, then this site's guides, then the directory's order.
 */
export function suggest(items, query, limit = 6) {
  const ts = terms(query);
  if (!ts.length) return [];
  return items
    .map((it) => {
      const title = normalize(it.name).split(' ');
      const words = normalize(it.text).split(' ');
      let score = 0;
      for (const t of ts) {
        if (title.some((w) => w.startsWith(t))) score += 3;
        else if (words.some((w) => w.startsWith(t))) score += 2;
        else if (words.some((w) => near(w, t))) score += 1;
        else return null;
      }
      return { it, score: score + (it.ours ? 0.5 : 0) };
    })
    .filter(Boolean)
    .sort((a, b) => b.score - a.score || a.it.i - b.it.i)
    .slice(0, limit)
    .map((r) => r.it);
}
