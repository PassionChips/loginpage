# India Guide

Plain-language guides to Indian government procedures (passport, Aadhaar, PAN and so on), in 12 languages: English, Hindi, Bengali, Marathi, Telugu, Tamil, Gujarati, Urdu, Kannada, Odia, Malayalam and Punjabi. Free to use, no accounts, funded by ads later.

Every guide links to the official source and shows when it was last checked. The site explains; it never files, collects documents or takes payments. That is deliberate: it keeps legal and privacy risk low.

## Run it

```sh
cargo run --bin run_main        # http://127.0.0.1:3000
cargo test
```

Settings come from environment variables:

| Variable | Default | Purpose |
| --- | --- | --- |
| `SITE_URL` | `http://localhost:3000` | Public address, used in canonical links and the sitemap |
| `SITE_NAME` | `India Guide` | Name in the header and page titles |
| `SITE_CONTACT_EMAIL` | `contact@example.com` | Target of the "report an error" link. **Set before launch.** |
| `BIND_ADDR` | `127.0.0.1:3000` | Listen address |

## Languages

- The interface text lives in `content/ui/<code>.txt`, one `key = value` per line. English must define every key (the server refuses to start otherwise); any key another language leaves out falls back to English. Unknown keys are rejected, which catches typos.
- Guides live in `content/guides/<code>/<slug>.md`. A guide with no translation is shown in English, with a notice, so every link works. Those fallback pages point search engines at the English original and are left out of the sitemap.
- First-time visitors are sent to the language their browser asks for (`Accept-Language`), English otherwise. Urdu is laid out right to left.
- The server logs a `translation gap` line at startup for each language with missing text or guides.
- To add a language: add a row to `LANGS` in `src/content.rs` and a `content/ui/<code>.txt` file.
- **The interface text and the Hindi guides were machine-written. Have a native speaker proofread each language before launch**, especially anything about fees, documents or deadlines.

## Add a guide

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

## Writing rules (keep risk low)

- Describe the process; send people to the official site to act. Do not promise outcomes.
- Cite the official source and keep `updated` honest. Re-check the guide when you change it, and remove it if you cannot keep it current.
- Write in your own words. Do not copy other sites or official text.
- No government logos or look-alike names. Do not collect personal data.

## Before launch

- [ ] Verify every guide against the official portal and update `updated`.
- [ ] Have native speakers proofread every language's interface text and guides.
- [ ] Have a lawyer review `content/pages/` (disclaimer, terms, privacy). They are drafts.
- [ ] Set `SITE_URL` and `SITE_CONTACT_EMAIL`.
- [ ] Put the site behind HTTPS (for example a reverse proxy) before exposing it.
- [ ] Before adding ads, update the privacy page, add consent where required, and relax the Content-Security-Policy in `src/lib.rs` for the ad provider only.

## Layout

```
content/guides/<lang>/    guides (Markdown + front matter)
content/ui/<lang>.txt     interface text per language
content/pages/            about, disclaimer, terms, privacy
static/css/site.css       styles
src/content.rs            loads and validates content
src/render.rs             HTML templates and interface text per language
src/routes.rs             routes, sitemap, robots
src/middleware/           per-IP rate limiting
```
