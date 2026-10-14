// Pure filtering and sorting rules for the directory. No DOM access, so they are unit-tested in Node.

export const MAX_COMPARE = 3;
/** Tools per category on the main page before the visitor searches or filters (matches the server). */
export const PREVIEW_PER_CATEGORY = 8;

/** Lower-cases and turns punctuation into spaces, so "(official" matches a search for "official". */
export function normalize(text) {
  return String(text).toLowerCase().replace(/[^\p{L}\p{N}\p{M}]+/gu, ' ').trim();
}

export function terms(query) {
  return normalize(query).split(' ').filter(Boolean);
}

/** True when a tool passes every active filter. Each search word must match the start of a word. */
export function matches(tool, filters, saved) {
  if (filters.badges.size && !filters.badges.has(tool.badge)) return false;
  if (filters.savedOnly && !saved.has(tool.url)) return false;
  const haystack = ' ' + tool.text;
  return terms(filters.query).every((t) => haystack.includes(' ' + t));
}

const byName = (a, b) => a.name.localeCompare(b.name, undefined, { sensitivity: 'base' });

/** Sort orders the data really supports. "curated" is the order the directory file lists them in. */
export function sorter(mode) {
  if (mode === 'az') return (a, b) => byName(a, b) || a.i - b.i;
  if (mode === 'za') return (a, b) => byName(b, a) || a.i - b.i;
  // GitHub stars as recorded by tools/fetch_stars.py; tools without a repository keep their order after.
  if (mode === 'stars') return (a, b) => (b.stars || 0) - (a.stars || 0) || a.i - b.i;
  return (a, b) => a.i - b.i;
}

/** Adds or removes `item`, refusing to grow past `max`. Returns the new list and what happened. */
export function toggleLimited(list, item, max) {
  if (list.includes(item)) return { list: list.filter((x) => x !== item), result: 'removed' };
  if (list.length >= max) return { list, result: 'full' };
  return { list: [...list, item], result: 'added' };
}

/** Command palette results: matching categories first, then tools whose name starts with the query. */
export function paletteResults(tools, categories, query, limit = 8) {
  const ts = terms(query);
  if (!ts.length) return categories.map((c) => ({ kind: 'category', item: c }));
  const cats = categories
    .filter((c) => ts.every((t) => (' ' + normalize(c.name + ' ' + c.id)).includes(' ' + t)))
    .map((c) => ({ kind: 'category', item: c }));
  const scored = tools
    .filter((tool) => ts.every((t) => (' ' + tool.text).includes(' ' + t)))
    .map((tool) => {
      const name = normalize(tool.name);
      const score = name.startsWith(ts.join(' ')) ? 0 : ts.every((t) => (' ' + name).includes(' ' + t)) ? 1 : 2;
      return { tool, score };
    })
    .sort((a, b) => a.score - b.score || a.tool.i - b.tool.i)
    .map(({ tool }) => ({ kind: 'tool', item: tool }));
  return [...cats, ...scored].slice(0, limit);
}

/**
 * Which matching tools of one category to show. On the main page, while nothing is filtered, only the
 * first few (in the current sort order) show; any search or filter reveals every match.
 */
export function visibleInCategory(sortedMatches, { preview, filtered }) {
  return preview && !filtered ? sortedMatches.slice(0, PREVIEW_PER_CATEGORY) : sortedMatches;
}
