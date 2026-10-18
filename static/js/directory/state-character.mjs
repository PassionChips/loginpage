// State characters on the India map: hovering (or focusing) a state that has one makes its small character
// grow up out of the state's outline, hold its pose or play its movement, and sink back into it. Every new
// hover plays it again. A mouse click or Enter opens the state as usual; on touch screens (no hover) the first
// tap plays it and the second opens the page. Nothing plays with reduced motion; taps then open at once.
//
// A character is either a still image (a transparent WebP) or a transparent animation: a VP9 WebM with
// alpha, or an animated WebP on Apple's WebKit, whose WebM playback drops the alpha.

/** The states that have a character, by the map's region id. `duration` is how long it stays out (ms). */
export const CHARACTERS = {
  'tamil-nadu': { animation: '/static/media/tamil-nadu-character', duration: 2133 },
  kerala: { image: '/static/media/kerala-character.webp', duration: 1600 },
};

const SINK = 380; // ms, matches the char-sink animation in site.css

// Only one character is out at a time: neighbouring states (Tamil Nadu and Kerala) would overlap. Bringing
// one out sends the one already out back into its state first.
let active = null;

export function setupStateCharacter(wrap, region, character) {
  const reduceMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches;
  const { animation, image, duration } = character;
  const probe = document.createElement('video');
  const useWebp = Boolean(animation) && (/Apple/.test(navigator.vendor || '') || !(probe.canPlayType && probe.canPlayType('video/webm; codecs="vp9"')));

  let pop = null;
  let media = null;
  let webpBlob = null;
  let webpUrl = null;
  let busy = false;
  let sinkNow = null; // ends the current showing early
  const me = { dismiss: () => sinkNow && sinkNow() };

  // Built (and its file downloaded) the first time the pointer reaches the map, not on page load.
  function build() {
    if (pop) return;
    pop = document.createElement('div');
    pop.className = 'char-pop';
    pop.dataset.state = region.dataset.id;
    pop.hidden = true;
    pop.setAttribute('aria-hidden', 'true');
    if (image) {
      media = document.createElement('img');
      media.alt = '';
      media.decoding = 'async';
      media.src = image;
    } else if (useWebp) {
      media = document.createElement('img');
      media.alt = '';
      // An animated image only restarts as a fresh copy, so keep the file to make one per hover.
      fetch(animation + '.webp')
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
      media.poster = animation + '-poster.webp';
      media.src = animation + '.webm';
    }
    media.className = 'char-pop-media';
    pop.append(media);
    wrap.append(pop);
  }

  // Stand the character on its state: its feet at the state's middle, wherever the map is drawn now.
  function place() {
    const box = wrap.getBoundingClientRect();
    const st = region.getBoundingClientRect();
    pop.style.left = `${st.left + st.width / 2 - box.left}px`;
    pop.style.top = `${st.top + st.height * 0.6 - box.top}px`;
  }

  function play() {
    if (busy || reduceMotion()) return;
    if (active && active !== me) active.dismiss();
    active = me;
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
      sinkNow = null;
      clearTimeout(guard);
      if (media.pause) media.pause();
      pop.classList.replace('is-out', 'is-in');
      region.classList.remove('is-portal');
      setTimeout(() => {
        pop.hidden = true;
        pop.classList.remove('is-in');
        busy = false;
        if (active === me) active = null;
      }, SINK);
    };
    sinkNow = sink;
    // Whatever the media does, the character never stays out.
    const guard = setTimeout(sink, duration + 1500);

    if (image) {
      setTimeout(sink, duration);
      return;
    }
    if (useWebp) {
      if (webpBlob) {
        if (webpUrl) URL.revokeObjectURL(webpUrl);
        webpUrl = URL.createObjectURL(webpBlob);
        media.src = webpUrl;
      } else {
        media.src = animation + '.webp'; // first hover, before the copy has arrived: never shown yet, so it starts at the beginning
      }
      setTimeout(sink, duration);
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

  // Touch screens have no hover: the first tap on the state plays the character and says "tap again to
  // open"; a second tap while it is out, or soon after, opens the page as usual.
  const tip = wrap.querySelector('.imap-tip');
  const tapAgain = wrap.querySelector('.imap')?.dataset.tapAgain || '';
  let lastPointer = 'mouse';
  let armedUntil = 0;
  let hintTimer = null;
  region.addEventListener('pointerdown', (e) => (lastPointer = e.pointerType));
  region.addEventListener('touchstart', () => build(), { passive: true });
  region.addEventListener('click', (e) => {
    if (lastPointer !== 'touch' || reduceMotion() || Date.now() < armedUntil) return;
    e.preventDefault();
    armedUntil = Date.now() + duration + 3000;
    play();
    // The map hides its tooltip when the finger lifts, so show the hint just after that.
    setTimeout(() => {
      if (!tip || !tapAgain) return;
      const box = wrap.getBoundingClientRect();
      const st = region.getBoundingClientRect();
      tip.textContent = `${region.dataset.name} · ${tapAgain}`;
      tip.classList.remove('is-off');
      tip.style.setProperty('--x', `${st.left + st.width / 2 - box.left}px`);
      tip.style.setProperty('--y', `${st.bottom - box.top + 44}px`);
      tip.hidden = false;
      clearTimeout(hintTimer);
      hintTimer = setTimeout(() => (tip.hidden = true), armedUntil - Date.now());
    }, 0);
  });
  return { play };
}
