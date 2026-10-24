# guideboy

Plain-language guides to Indian government procedures (passport, Aadhaar, PAN and so on), in 12 languages: English, Hindi, Bengali, Marathi, Telugu, Tamil, Gujarati, Urdu, Kannada, Odia, Malayalam and Punjabi. Free to use, no accounts, funded by ads later.

Every guide links to the official source and shows when it was last checked. The site explains; it never files, collects documents or takes payments. That is deliberate: it keeps legal and privacy risk low.

## Run it

```sh
cargo run --bin run_main        # http://127.0.0.1:3000
cargo test                      # server and page tests
node --test tests/js/*.test.mjs # directory filters, journey stages and search suggestions
```

Settings come from environment variables:

| Variable | Default | Purpose |
| --- | --- | --- |
| `SITE_URL` | `http://localhost:3000` | Public address, used in canonical links and the sitemap |
| `SITE_NAME` | `guideboy` | Name in the header and page titles |
| `SITE_CONTACT_EMAIL` | `guideboy20@gmail.com` | Target of the "report an error" link. |
| `BIND_ADDR` | `127.0.0.1:3000` | Listen address |

## Languages

- The interface text lives in `content/ui/<code>.txt`, one `key = value` per line. English must define every key (the server refuses to start otherwise); any key another language leaves out falls back to English. Unknown keys are rejected, which catches typos.
- Guides live in `content/guides/<code>/<slug>.md`. A guide with no translation is shown in English, with a notice, so every link works. Those fallback pages point search engines at the English original and are left out of the sitemap.
- First-time visitors are sent to the language their browser asks for (`Accept-Language`), English otherwise. Urdu is laid out right to left.
- The server logs a `translation gap` line at startup for each language with missing text or guides.
- To add a language: add a row to `LANGS` in `src/content.rs` and a `content/ui/<code>.txt` file.
- **The interface text and the Hindi guides were machine-written. Have a native speaker proofread each language before launch**, especially anything about fees, documents or deadlines.

## Guides directory (states)

`/<lang>/guides` is laid out exactly like the free tools directory (same markup, script and styles): a rail of sections (All India, then one per state), each section previewing its first 8 entries, with search, topic filters, bookmarks, details and compare. Each state also has its own page, `/<lang>/guides/<state>`, where its entries are grouped by topic. The data is one file, `content/guides/directory.txt`:

```
[tamil-nadu]
jobs | TNPSC | https://www.tnpsc.gov.in | Group 1, Group 2/2A, Group 4 and other notified posts.
[india]
id | guide=passport-new
```

- Sections are listed in `GUIDE_STATES` and topics (`jobs`, `certificates`, `scholarships`, `admissions`, `welfare`, `land`, `citizen`, `id`) in `GUIDE_TOPICS`, both in `src/content.rs`. Their names are translated in `content/ui/<lang>.txt` (`st_*`, `gt_*`); the page text uses `gx_*`. To add a state: add its id to `GUIDE_STATES`, an `st_<id>` line to every language file, and a `[<id>]` section to the file.
- An entry is either an official portal (`topic | Name | https url | description`, opens in a new tab) or one of this site's guides (`topic | guide=<slug>`, shown with the guide's own title and summary in the reader's language).
- The server refuses to start on an unknown state or topic, a non-https link, a URL listed twice, a missing guide, an empty state, or a guide whose slug is a state id.
- Cards show a service icon (each of this site's guides has its own, see `service_icon` in `src/render.rs`; portals show their topic's icon) and where the entry applies: the true outline of India or of the state. A state's page also marks the state on the map of India.
- The outlines come from [India boundaries](https://github.com/datameet/maps) by the DataMeet India community (Survey of India based, CC BY 4.0, credited on the page). `python3 -I tools/build_geo.py DATA_DIR` turns `States/Admin2.shp`/`.dbf` and `Country/india-composite.geojson` from that repository into `static/geo/outlines.svg` (all 28 states and 8 union territories, plus India) and `src/geo.rs`. Never draw or edit outlines by hand.
- **Only official government portals.** Several state portals did not answer from outside India when this list was made (October 2026); check each link from an Indian connection before launch.

### State characters (Tamil Nadu, Kerala)

On the India map, hovering (or keyboard-focusing) Tamil Nadu or Kerala makes that state's small cut-out character grow up out of the state and sink back into it; every new hover plays it again. Tamil Nadu's character does a hand movement (an animation); Kerala's is a still image that stays out for 1.6 s. A mouse click or Enter opens the state's page like any other state. Touch screens have no hover, so there the first tap plays it (with a "tap again to open" hint, `gx_tap_again`) and a second tap opens the page. With reduced motion nothing plays and a tap opens the page at once. The code is `static/js/directory/state-character.mjs`; to give another state a character, add it to `CHARACTERS` there (`image` for a still transparent WebP, `animation` for a WebM/WebP pair) and put the file in `static/media/`. Files download only once the pointer reaches the map. Safari replays the animated WebP from a fresh `blob:` copy each time, which is why the CSP allows `blob:` images.

`static/media/kerala-character.webp` is the supplied `kerala.png` cropped to the figure (169×459, transparent).

The character is the supplied animation with its background removed (rembg `isnet-general-use` on every frame, then only the largest shape kept, edges tightened and de-tinted, and the left and right frame cuts faded out), at 405×720:
`static/media/tamil-nadu-character.webm` (VP9 with alpha, 30 fps; Chrome, Edge, Firefox, Android), `tamil-nadu-character.webp` (animated WebP played once, 15 fps, 360×640; Safari and iOS, whose WebM playback drops alpha) and `tamil-nadu-character-poster.webp` (first frame). From a folder of transparent PNG frames: `ffmpeg -framerate 30 -i %03d.png -c:v libvpx-vp9 -pix_fmt yuva420p -b:v 0 -crf 30 -auto-alt-ref 0 -an tamil-nadu-character.webm` and `ffmpeg -framerate 30 -i %03d.png -vf fps=15,scale=360:640 -c:v libwebp_anim -quality 60 -loop 1 tamil-nadu-character.webp`. If the animation's length changes, update `DURATION` in the script.

### TRB updates (automatic)

Tamil Nadu › Jobs has a **TRB: what's new** card (`/<lang>/guides/tamil-nadu/trb`), and the TRB guide shows the latest five. Every item in the "What's New" list of trb.tn.gov.in gets its own page (`/<lang>/guides/tamil-nadu/trb/<id>`): TRB's date and title, the type of update (result, answer key, selection list...), the exam or recruitment, the notification number, a next step in our own words, and links to TRB's notice and PDFs. TRB's announcement text is never copied.

- `tools/fetch_trb.py` (Python, standard library only) reads TRB's list and each notice page and writes `content/trb/updates.txt`. Run it by hand with `python3 -I tools/fetch_trb.py`. Old items stay as an archive (up to 60). If TRB's page cannot be read, it stops with an error and leaves the file alone.
- `.github/workflows/trb-updates.yml` runs it every 12 hours and commits the file to `main` only when something changed; Cloudflare then publishes the site as for any commit. Disable the workflow in GitHub's Actions tab to stop it.
- The type and exam come from words in TRB's titles (`KINDS` and `EXAMS` in the script); add a pattern there, plus `tk_`/`tn_`/`te_` interface strings, for a new kind of update. The server refuses to start if the file has an unknown type, a bad date or a non-https link.
- **MRB** works the same way: `tools/fetch_mrb.py` reads the Notifications page of mrb.tn.gov.in into `content/mrb/updates.txt` (one page per recruitment at `/<lang>/guides/tamil-nadu/mrb/<id>`, with all its documents). MRB gives no dates, so a recruitment moves to the top when the script first sees it or sees a new document in it. The same workflow runs both scripts; if one board's site fails, the other's list is still committed and the run is marked failed.
- **AP DSC** (Andhra Pradesh) works the same way: `tools/fetch_apdsc.py` reads the document lists of tet2dsc.apcfss.in (DSC, APTET, Special TET) from the site's script into `content/apdsc/updates.txt`, one page per programme at `/<lang>/guides/andhra-pradesh/apdsc/<id>`. The site resolves to IPv6 first, so the script tries IPv4 addresses first.
- Each board belongs to a state (`state` in `BOARDS`): its pages live under that state's directory page and take its colour.
- Boards are listed in `BOARDS` (`src/content.rs`); a guide shows a board's latest items with `feed: trb` or `feed: mrb` in its front matter.
- **TNEA** has a how-to-apply guide (`tnea-engineering-admission`) but no automatic updates: tneaonline.org shows its announcements only after login.
- Directory entries can point to this site's own pages listed in `INTERNAL_PAGES` (`src/content.rs`), as `topic | Name | /guides/... | description`.

## Guided mode

Every guide page has two reading modes: **Guided** (the default, one stage at a time) and **Full guide** (the article). The guided view is built in the browser (`static/js/guide/journey.mjs`) from the guide's own `##` sections, so there is nothing extra to write; visitors without script, search engines and printouts get the full article. The section id decides the stage (`static/js/guide/plan.mjs`):

- `{#situation}`: optional "which applies to you?" question. Write it as a list of links: `- [I failed my test](#problems)` jumps to that section in guided mode (and works as a jump list in the full guide); a link to another site opens it.
- `{#before}`, `{#eligibility}`, `{#fee}`: "Check you can apply".
- `{#documents}`: a checklist; each top-level list item gets a tick box.
- `{#steps}` and any other section with a numbered list: a timeline with a tick per step.
- everything else (`{#after}`, `{#help}`, `{#problems}`...): cards to open on the last stage, after the official site.

Ticks, the mode and the stage are kept in the visitor's browser only. `content/guides/glossary.txt` explains short forms (RTO, OTP...) on English guides: the first use of each becomes a button that shows its meaning.

The guides page (`/<lang>/guides`) starts with the question "What do you need help getting done?", search suggestions, this site's guides as task cards with their step and document counts (counted from the guide), category tabs (from each guide's `category`) and the guides in progress in this browser.

## Add a guide

Every guide is listed in the guides directory (add it to `content/guides/directory.txt`) and lives at `/<lang>/guides/<slug>`. The front page (`/<lang>`) only links to the list.

Create `content/guides/en/<slug>.md`, then add translations as the same `<slug>.md` under the other language folders:

```md
---
title: How to ...
summary: One sentence for the list and search results.
category: Identity documents
official_name: Name of the department or portal
official_url: https://official.site.gov.in
updated: 2026-10-14
---

Body in Markdown.
```

The slug is the file name (lowercase letters, digits, `-`). The server refuses to start if front matter is missing or malformed, `official_url` is not `https://`, or `updated` is not `YYYY-MM-DD`. Raw HTML in the body is shown as text and `javascript:` links are removed.

Every guide needs a `## Documents you will need {#documents}` section (a test checks it, and the toolbar's Documents button jumps there). Write steps only from what the official portal, its user manual, FAQ or the law says; leave out anything you cannot find there.

### Guide screenshots

An image alone on its own line becomes a captioned figure. The title in quotes is the caption and must name the source and date:

```md
1. Under **Quick Links**, click **Instant E-PAN**.

   ![Quick Links with "Instant E-PAN" outlined](/static/guides/pan-instant-epan/1-quick-links.webp "Screenshot of incometax.gov.in, 14 Oct 2026. The red outline is ours.")
```

- Images must live in `static/guides/<slug>/`; any other image source is dropped. A test checks that every screenshot exists and has a "Screenshot of …" caption.
- Take them with `FMT=webp node tools/screenshot.mjs URL static/guides/<slug>/<n>-<name>` (see the options at the top of the file: outline a button with `HL`, crop with `CROP`, click through with `CLICK`/`STEPS`). Crop to the part that matters so it is readable on a phone.
- **Only public, pre-login screens with no personal data**, and **only from portals whose copyright policy allows reuse with credit**. Checked October 2026:

| Portal | Screenshots |
| --- | --- |
| incometax.gov.in (Income Tax e-Filing) | Allowed, source must be acknowledged |
| rtionline.gov.in (RTI Online) | Allowed, source must be acknowledged |
| pgportal.gov.in (CPGRAMS) | Allowed, source must be acknowledged |
| passportindia.gov.in (Passport Seva, MEA) | Not without MEA's permission |
| parivahan.gov.in / Sarathi | Only after emailing the Web Information Manager for permission |
| scholarships.gov.in (NSP) | Not without permission |

Check a portal's copyright policy before adding its first screenshot, and add it to this table.

### Sources this machine could not reach

Some official portals block automated or non-Indian traffic: voters.eci.gov.in, the UIDAI FAQ pages, EPFO, e-Shram registration, ABHA, PM-JAY, TNPSC, TN e-Sevai, myScheme scheme pages and others. Guides for those services need to be researched from a normal browser in India.

## Free tools directory

`/<lang>/links` is a searchable directory of free tools (PDF, invoicing, AI, video, audio, design, files, developer tools, learning, and official anime and manga services). The data is one file, `content/links/directory.txt`:

```
[pdf]
Name | https://example.com | free | One-line English description.
Name | https://example.com | oss | One-line English description. | github=owner/repo
```

- Badges: `free` (free to use), `tier` (free plan with limits), `oss` (open source).
- The main page (`/<lang>/links`) shows the first 8 tools of each category, in file order, with a "See all" link; every category also has its own page (`/<lang>/links/<category>`) listing all of them. Search and filters on the main page still cover every tool. Put the strongest tools first in each group.
- Open-source tools can name their GitHub repository with `github=owner/repo`. `python3 -I tools/fetch_stars.py` copies each repository's current star count into `content/links/github.tsv` with the date checked; cards show it and the "Most GitHub stars" sort uses it. Never edit that file by hand. Re-run it before releases. It also reports archived repositories.
- The open-source tools added in October 2026 were chosen from GitHub data: at least 5,000 stars, not archived, and updated in the last 12 months. Keep to that bar when adding more.
- The server refuses to start on an unknown group, a non-https link, a duplicate URL, a bad badge or an empty group.
- Group names and the page text are translated in `content/ui/<lang>.txt` (`cat_*`, `links_*`, `badge_*`). Tool names and descriptions stay in English.
- **Only list legal services.** Do not add sites that distribute copyrighted anime, manga or films without permission, even if other directories do. Check that a "free" claim is still true before you add or keep a link.
- Tool icons are the sites' own icons, downloaded once into `static/icons/` by `python3 -I tools/fetch_icons.py` and served by this site, so visitors never contact the listed sites or a favicon service. Re-run it after adding or changing links: it keeps only square raster images, never stores SVG (SVG-only icons are drawn to PNG in headless Chrome), and deletes icons for removed tools. Tools without an icon show a letter tile.
- Each directory page has a background photo in `static/backgrounds/` (`all.jpg` for the main page, `<category>.jpg` for each category), listed with its credit in `content/links/backgrounds.tsv`; full records are in `static/backgrounds/credits.json`. Only CC0 or public-domain photos are used, and each page shows its credit anyway. Photos were found with the design skill's `assets.mjs` (Openverse and Wikimedia Commons) and compressed to about 1600px wide.
- External links open in a new tab with `rel="nofollow noopener noreferrer"`. No favicons or images are loaded from the listed sites.
- With script, the page adds: debounced search (`/` to focus), a command palette (Ctrl/Cmd + K), filters by badge and saved tools, A–Z / Z–A sorting, grid/list and density views, bookmarks, a details dialog, and comparing up to three tools. Without script every tool is still listed and linked.
- The directory script reads the tools from the rendered cards, so `directory.txt` is the only source of tool data. Filters, sorts and comparison rows only use fields that file has (name, link, category, badge, description). Add a field there before adding a filter or comparison row for it.
- Bookmarks are stored in the visitor's browser (`localStorage`), and the comparison list and view choice for the current session (`sessionStorage`). Nothing is sent to the server. The Privacy page says so.
- The code is in `static/js/directory/`: `filters.mjs` (pure rules, unit-tested), `saved.mjs`, `compare.mjs`, `details.mjs`, `palette.mjs`, `rail.mjs`, `toast.mjs`, wired together by `main.mjs`.

## Fonts

Headlines use Newsreader, labels and buttons JetBrains Mono, and the dot-matrix wordmark and footer links Doto (round dots via its `ROND` axis). All three are SIL Open Font License fonts, self-hosted in `static/fonts/` with their licence files (`OFL-*.txt`), so visitors never contact Google Fonts. They only cover Latin letters: on Indian-language pages headings keep the bold sans font, and the dotted footer links fall back to it.

## Writing rules (keep risk low)

- Describe the process; send people to the official site to act. Do not promise outcomes.
- Cite the official source and keep `updated` honest. Re-check the guide when you change it, and remove it if you cannot keep it current.
- Write in your own words. Do not copy other sites or official text.
- No government logos or look-alike names. Do not collect personal data.

## Deploy (Cloudflare, free)

The site is published as static files: `cargo run --release --bin export -- public` writes every page to `public/` as `<path>.html` (`/en/guides` -> `public/en/guides.html`), copies `static/`, and adds `sitemap.xml`, `robots.txt`, a `_headers` file (the same security headers the server sends, plus caching for `/static/*`) and a `_redirects` file (`/` -> `/en`). `wrangler.jsonc` tells Cloudflare to serve that folder with clean URLs and `404.html` for unknown paths. No Worker code runs. A test exports the site and checks every internal link, image, script and stylesheet points at a real file.

In the Cloudflare dashboard (Workers & Pages -> Create -> import the GitHub repository):

| Field | Value |
| --- | --- |
| Build command | `bash tools/cloudflare-build.sh` |
| Deploy command | `npx wrangler deploy` |
| Non-production branch deploy command | `npx wrangler preview` (uses the `previews` block in `wrangler.jsonc`) |
| Build variable | `SITE_URL` = the public https address, e.g. `https://guideboy.fyi` (or the `*.workers.dev` address until the domain is set up) |

Cloudflare's build machine has no Rust, so the build script installs it first (the first build takes a few minutes). The export refuses to run without an `https://` SITE_URL, because canonical links, hreflang, the sitemap and robots.txt are built from it. To try the export locally: `EXPORT_ALLOW_LOCAL=1 SITE_URL=http://localhost:8787 cargo run --bin export -- public`, then `npx wrangler dev`.

Differences from the server: `/` always goes to `/en` (the server picks the browser's language), and there is no per-IP rate limit (Cloudflare absorbs traffic). The Rust server still works for local development and could be hosted on a VPS later if the site ever needs server features.

## Before launch

- [ ] Verify every guide against the official portal and update `updated`.
- [ ] Have native speakers proofread every language's interface text and guides.
- [ ] Have a lawyer review `content/pages/` (disclaimer, terms, privacy). They are drafts.
- [ ] Set the `SITE_URL` build variable in Cloudflare to the real https address.
- [ ] Before adding ads, update the privacy page, add consent where required, and relax the Content-Security-Policy in `src/lib.rs` for the ad provider only.

## Layout

```
content/guides/<lang>/    guides (Markdown + front matter)
content/ui/<lang>.txt     interface text per language
content/pages/            about, disclaimer, terms, privacy
content/links/            free tools directory data
static/css/site.css       styles
src/content.rs            loads and validates content
src/render.rs             HTML templates and interface text per language
src/routes.rs             routes, sitemap, robots
src/middleware/           per-IP rate limiting
```
