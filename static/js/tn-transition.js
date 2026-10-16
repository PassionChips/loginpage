// The Tamil Nadu page transition: opening the Tamil Nadu guides page from anywhere on the site first pops
// up the small Tamil Nadu character, cut out over the page, doing its hand movement; then goes there. Only Tamil Nadu has one; every other link and
// state navigates exactly as before.
//
// One function decides and runs it (`guideboyNavigate`), used by a single click listener for all links
// (rail, headings, "see all", map regions, the state list, breadcrumbs) and by the scripts that navigate
// in code (command palette, suggestions, Space on the map). The site is static pages, so the last step is
// an ordinary page load; the Tamil Nadu page then fades in from the overlay's colour.
//
// The character is a transparent animation: a VP9 WebM with alpha where browsers support that, and an
// animated WebP (played once) on Apple's WebKit, whose WebM playback drops the transparency.
//
// It never gets in the way: reduced motion, data saver, a modified click, a missing or failing animation, or
// a slow start all navigate straight away, and a timer guarantees the overlay never outlives the animation.
(function () {
  'use strict';

  var MEDIA = '/static/media/tamil-nadu-character';
  var DURATION = 2133; // ms, the length of the character's animation
  var REVEAL_KEY = 'guideboy:tn-reveal';
  var START_TIMEOUT = 1200; // ms to wait for the first frame before giving up and navigating
  var MAX_TOTAL = 5000; // ms after which the overlay always navigates, whatever the animation does

  /** True for the Tamil Nadu guides page in any language: /en/guides/tamil-nadu, /ta/guides/tamil-nadu … */
  function isTamilNaduPage(path) {
    return /^\/[a-z]{2}\/guides\/tamil-nadu\/?$/.test(path);
  }

  /** Whether following `href` from `fromPath` should play the transition. */
  function wantsTransition(href, fromPath, origin) {
    var url;
    try {
      url = new URL(href, origin);
    } catch (e) {
      return false;
    }
    // Switching the Tamil Nadu page's language, or reloading it, is not "opening Tamil Nadu".
    return url.origin === origin && isTamilNaduPage(url.pathname) && !isTamilNaduPage(fromPath);
  }

  var api = { isTamilNaduPage: isTamilNaduPage, wantsTransition: wantsTransition };
  globalThis.guideboyTransition = api;
  if (typeof document === 'undefined') return; // loaded by the unit tests

  var root = document.documentElement;
  var reduceMotion = function () {
    return window.matchMedia && matchMedia('(prefers-reduced-motion: reduce)').matches;
  };
  var saveData = function () {
    return Boolean(navigator.connection && navigator.connection.saveData);
  };

  // Arriving on the Tamil Nadu page from the transition: start under the overlay's colour and fade in.
  try {
    if (sessionStorage.getItem(REVEAL_KEY)) {
      sessionStorage.removeItem(REVEAL_KEY);
      if (isTamilNaduPage(location.pathname) && !reduceMotion()) root.classList.add('tn-reveal');
    }
  } catch (e) {}

  var busy = false;
  var overlay = null;
  var actor = null; // the character: a <video> (WebM with alpha) or an <img> (animated WebP)

  // WebKit (Safari, and every browser on iOS) plays WebM but without its alpha channel.
  var useWebp = function () {
    if (/Apple/.test(navigator.vendor || '')) return true;
    var probe = document.createElement('video');
    return !(probe.canPlayType && probe.canPlayType('video/webm; codecs="vp9"'));
  };

  // Load the character ahead of time when a visitor is about to choose Tamil Nadu, not on every page view.
  function warm() {
    if (actor || reduceMotion() || saveData()) return;
    if (useWebp()) {
      // Only warm the cache: the image shown is created on play, so it animates from its first frame.
      new Image().src = MEDIA + '.webp';
      actor = document.createElement('img');
      actor.alt = '';
    } else {
      actor = document.createElement('video');
      actor.muted = true;
      actor.defaultMuted = true;
      actor.playsInline = true;
      actor.setAttribute('muted', '');
      actor.setAttribute('playsinline', '');
      actor.preload = 'auto';
      actor.poster = MEDIA + '-poster.webp';
      actor.src = MEDIA + '.webm';
    }
    actor.className = 'tn-char';
    actor.setAttribute('aria-hidden', 'true');
  }

  function go(href) {
    location.assign(href);
  }

  function play(href) {
    busy = true;
    warm();
    try {
      sessionStorage.setItem(REVEAL_KEY, '1');
    } catch (e) {}
    // Fetch the page while the animation plays, so it appears as soon as it ends.
    var hint = document.createElement('link');
    hint.rel = 'prefetch';
    hint.href = href;
    document.head.appendChild(hint);

    overlay = document.createElement('div');
    overlay.className = 'tn-overlay';
    overlay.setAttribute('role', 'dialog');
    overlay.setAttribute('aria-label', 'Tamil Nadu');
    var glow = document.createElement('div');
    glow.className = 'tn-glow';
    overlay.appendChild(glow);
    overlay.appendChild(actor);
    document.body.appendChild(overlay);
    root.classList.add('tn-playing');

    var done = false;
    var finish = function () {
      if (done) return;
      done = true;
      clearTimeout(startTimer);
      clearTimeout(maxTimer);
      clearTimeout(endTimer);
      go(href);
    };
    var fail = function () {
      // Nothing was shown worth waiting for: skip the fade-in on arrival too.
      try {
        sessionStorage.removeItem(REVEAL_KEY);
      } catch (e) {}
      finish();
    };
    var startTimer = setTimeout(fail, START_TIMEOUT);
    var maxTimer = setTimeout(finish, MAX_TOTAL);
    var endTimer = null;
    var started = function () {
      clearTimeout(startTimer);
      overlay.classList.add('is-on');
    };

    // A click or Escape skips the rest, but not the second click of a double click on the link.
    var opened = Date.now();
    overlay.addEventListener('click', function () {
      if (Date.now() - opened > 600) finish();
    });
    document.addEventListener('keydown', function (e) {
      if (e.key === 'Escape') finish();
    });

    if (actor.tagName === 'IMG') {
      // An animated image has no "ended" event: it starts when shown, and ends after the animation's length.
      actor.addEventListener('load', function () {
        started();
        endTimer = setTimeout(finish, DURATION + 120);
      }, { once: true });
      actor.addEventListener('error', fail, { once: true });
      actor.src = MEDIA + '.webp';
      return;
    }
    actor.addEventListener('playing', started, { once: true });
    actor.addEventListener('ended', finish, { once: true });
    actor.addEventListener('error', fail, { once: true });
    try {
      actor.currentTime = 0;
      var playing = actor.play();
      if (playing && playing.catch) playing.catch(fail);
    } catch (e) {
      fail();
    }
  }

  /** Navigates to `href`, through the Tamil Nadu transition when it applies. Every entry point uses this. */
  function navigate(href) {
    if (busy) return;
    if (!wantsTransition(href, location.pathname, location.origin) || reduceMotion() || saveData() || !document.body) {
      go(href);
      return;
    }
    play(href);
  }
  api.navigate = navigate;
  window.guideboyNavigate = navigate;

  var linkFor = function (target) {
    var a = target && target.closest ? target.closest('a[href]') : null;
    if (!a) return null;
    var href = a.getAttribute('href');
    return href && wantsTransition(href, location.pathname, location.origin) ? { a: a, href: href } : null;
  };

  // Every Tamil Nadu link on every page, including links added later by scripts.
  document.addEventListener('click', function (e) {
    var hit = linkFor(e.target);
    if (!hit) return;
    // Let the browser handle new tabs, downloads and other modified clicks.
    if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
    // getAttribute, not .target: on the map's SVG links that property is an object, not a string.
    var target = hit.a.getAttribute('target');
    if (target && target !== '_self') return;
    e.preventDefault();
    navigate(hit.href);
  }, true);
  ['pointerover', 'focusin', 'touchstart'].forEach(function (type) {
    document.addEventListener(type, function (e) {
      if (linkFor(e.target)) warm();
    }, { passive: true, capture: true });
  });

  // Back from Tamil Nadu may restore this page from the browser's cache: never with the overlay still up.
  window.addEventListener('pageshow', function (e) {
    if (!e.persisted) return;
    busy = false;
    root.classList.remove('tn-playing');
    if (overlay) overlay.remove();
    overlay = null;
    if (actor && actor.pause) actor.pause();
  });
})();
