# guideboy

Plain-language guides to Indian government procedures (passport, Aadhaar, PAN and so on), in 12 languages: English, Hindi, Bengali, Marathi, Telugu, Tamil, Gujarati, Urdu, Kannada, Odia, Malayalam and Punjabi. Free to use, no accounts, funded by ads later.

Every guide links to the official source and shows when it was last checked. The site explains; it never files, collects documents or takes payments. That is deliberate: it keeps legal and privacy risk low.

## Run it

```sh
cargo run --bin run_main        # http://127.0.0.1:3000
cargo test                      # server and page tests
node --test tests/js/*.test.mjs # directory filter and sort rules
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

## Add a guide

Guides are listed, searchable and filterable by category at `/<lang>/guides`; each one lives at `/<lang>/guides/<slug>`. The front page (`/<lang>`) only links to the list.

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

## Writing rules (keep risk low)

- Describe the process; send people to the official site to act. Do not promise outcomes.
- Cite the official source and keep `updated` honest. Re-check the guide when you change it, and remove it if you cannot keep it current.
- Write in your own words. Do not copy other sites or official text.
- No government logos or look-alike names. Do not collect personal data.

## Before launch

- [ ] Verify every guide against the official portal and update `updated`.
- [ ] Have native speakers proofread every language's interface text and guides.
- [ ] Have a lawyer review `content/pages/` (disclaimer, terms, privacy). They are drafts.
- [ ] Set `SITE_URL`.
- [ ] Put the site behind HTTPS (for example a reverse proxy) before exposing it.
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
