// Tamil Nadu's character on the India map: hovering (or focusing) Tamil Nadu makes the small character
// grow up out of the state's outline, do its hand movement, and sink back into it. Every new hover plays it
// again. Only Tamil Nadu has one, and it is decoration only: clicking Tamil Nadu navigates as usual.
//
// The character is a transparent animation: a VP9 WebM with alpha, or an animated WebP on Apple's WebKit,
// whose WebM playback drops the alpha. Nothing plays with reduced motion, or on touch (a tap navigates).
const MEDIA = '/static/media/tamil-nadu-character';
const DURATION = 2133; // ms, the length of the hand movement
const SINK = 380; // ms, matches the tn-sink animation in site.css

export function setupTamilNaduCharacter(wrap, region) {
  const reduceMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
  const probe = document.createElement('video');
  const useWebp = /Apple/.test(navigator.vendor || '') || !(probe.canPlayType && probe.canPlayType('video/webm; codecs="vp9"'));

  let pop = null;
  let media = null;
  let webpBlob = null;
  let webpUrl = null;
  let busy = false;

  // Built (and its animation downloaded) the first time the pointer reaches the map, not on page load.
  function build() {
    if (pop) return;
    pop = document.createElement('div');
    pop.className = 'tn-pop';
    pop.hidden = true;
    pop.setAttribute('aria-hidden', 'true');
    if (useWebp) {
      media = document.createElement('img');
      media.alt = '';
      // An animated image only restarts as a fresh copy, so keep the file to make one per hover.
      fetch(MEDIA + '.webp')
        .then((r) => (r.ok ? r.blob() : Promise.reject(r.status)))
        .then((b) => (webpBlob = b))
        .catch(() => {});
    } else {
      media = document.createElement('video');
      media.muted = true;
      media.defaultMuted = true;
      media.playsInline = true;
      media.setAttribute('muted', '');
      media.setAttribute('playsinline', '');
      media.preload = 'auto';
      media.poster = MEDIA + '-poster.webp';
      media.src = MEDIA + '.webm';
    }
    media.className = 'tn-pop-media';
    pop.append(media);
    wrap.append(pop);
  }

  // Stand the character on Tamil Nadu: its feet at the state's middle, wherever the map is drawn now.
  function place() {
    const box = wrap.getBoundingClientRect();
    const tn = region.getBoundingClientRect();
    pop.style.left = `${tn.left + tn.width / 2 - box.left}px`;
    pop.style.top = `${tn.top + tn.height * 0.6 - box.top}px`;
  }

  function play() {
    if (busy || reduceMotion()) return;
    build();
    busy = true;
    place();
    pop.hidden = false;
    pop.classList.remove('is-in');
    void pop.offsetWidth; // restart the CSS animation
    pop.classList.add('is-out');
    region.classList.add('is-portal');

    let done = false;
    const sink = () => {
      if (done) return;
      done = true;
      clearTimeout(guard);
      pop.classList.replace('is-out', 'is-in');
      setTimeout(() => {
        pop.hidden = true;
        pop.classList.remove('is-in');
        region.classList.remove('is-portal');
        if (media.pause) media.pause();
        busy = false;
      }, SINK);
    };
    // Whatever the media does, the character never stays out.
    const guard = setTimeout(sink, DURATION + 1500);

    if (useWebp) {
      if (webpBlob) {
        if (webpUrl) URL.revokeObjectURL(webpUrl);
        webpUrl = URL.createObjectURL(webpBlob);
        media.src = webpUrl;
      } else {
        media.src = MEDIA + '.webp'; // first hover, before the copy has arrived: never shown yet, so it starts at the beginning
      }
      setTimeout(sink, DURATION);
      return;
    }
    media.addEventListener('ended', sink, { once: true });
    media.addEventListener('error', sink, { once: true });
    try {
      media.currentTime = 0;
      const started = media.play();
      if (started && started.catch) started.catch(sink);
    } catch {
      sink();
    }
  }

  const fine = (e) => e.pointerType !== 'touch';
  wrap.addEventListener('pointerenter', (e) => fine(e) && !reduceMotion() && build());
  region.addEventListener('pointerenter', (e) => fine(e) && play());
  region.addEventListener('focus', play);
  return { play };
}
