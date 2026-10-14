// Browser storage that never throws. In private windows or with storage blocked it simply behaves as empty.
export function load(area, key, fallback) {
  try {
    const raw = globalThis[area]?.getItem(key);
    return raw == null ? fallback : JSON.parse(raw);
  } catch {
    return fallback;
  }
}

export function save(area, key, value) {
  try {
    globalThis[area]?.setItem(key, JSON.stringify(value));
  } catch {
    // Storage unavailable: the feature still works for this visit.
  }
}
