// Bookmarked tools, kept in this browser's local storage only. The site never receives the list.
import { load, save } from './store.mjs';

const KEY = 'guideboy:saved';

export function createSaved(validUrls) {
  const urls = new Set(load('localStorage', KEY, []).filter((u) => validUrls.has(u)));
  return {
    has: (url) => urls.has(url),
    get size() {
      return urls.size;
    },
    /** Returns true when the tool is now saved. */
    toggle(url) {
      const added = !urls.has(url);
      if (added) urls.add(url);
      else urls.delete(url);
      save('localStorage', KEY, [...urls]);
      return added;
    },
  };
}
