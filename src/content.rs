use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct LangInfo {
    pub code: &'static str,
    /// The language's name in its own script, shown in the language menu.
    pub native: &'static str,
    pub rtl: bool,
}

/// Supported languages, biggest first. English is index 0 and is the fallback for everything.
/// To add a language: add a row here and a `content/ui/<code>.txt` file.
pub const LANGS: [LangInfo; 12] = [
    LangInfo { code: "en", native: "English", rtl: false },
    LangInfo { code: "hi", native: "हिन्दी", rtl: false },
    LangInfo { code: "bn", native: "বাংলা", rtl: false },
    LangInfo { code: "mr", native: "मराठी", rtl: false },
    LangInfo { code: "te", native: "తెలుగు", rtl: false },
    LangInfo { code: "ta", native: "தமிழ்", rtl: false },
    LangInfo { code: "gu", native: "ગુજરાતી", rtl: false },
    LangInfo { code: "ur", native: "اردو", rtl: true },
    LangInfo { code: "kn", native: "ಕನ್ನಡ", rtl: false },
    LangInfo { code: "or", native: "ଓଡ଼ିଆ", rtl: false },
    LangInfo { code: "ml", native: "മലയാളം", rtl: false },
    LangInfo { code: "pa", native: "ਪੰਜਾਬੀ", rtl: false },
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Lang(usize);

impl Lang {
    pub const EN: Lang = Lang(0);

    pub fn all() -> impl Iterator<Item = Lang> {
        (0..LANGS.len()).map(Lang)
    }

    pub fn code(self) -> &'static str {
        LANGS[self.0].code
    }

    pub fn native_name(self) -> &'static str {
        LANGS[self.0].native
    }

    pub fn is_rtl(self) -> bool {
        LANGS[self.0].rtl
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        LANGS.iter().position(|l| l.code == code).map(Lang)
    }

    /// Picks the best supported language from an Accept-Language header, e.g. "hi-IN,hi;q=0.9,en;q=0.8".
    pub fn from_accept_language(header: &str) -> Option<Lang> {
        let mut prefs: Vec<(f32, &str)> = header
            .split(',')
            .filter_map(|part| {
                let mut pieces = part.trim().split(';');
                let tag = pieces.next()?.trim();
                let q = pieces
                    .find_map(|p| p.trim().strip_prefix("q=").and_then(|v| v.parse::<f32>().ok()))
                    .unwrap_or(1.0);
                (q > 0.0 && !tag.is_empty()).then_some((q, tag))
            })
            .collect();
        prefs.sort_by(|a, b| b.0.total_cmp(&a.0));
        prefs.into_iter().find_map(|(_, tag)| {
            let primary = tag.split('-').next().unwrap_or(tag).to_ascii_lowercase();
            Lang::from_code(&primary)
        })
    }
}

/// Every interface string the templates use. English must define all of them; other languages
/// may leave some out and fall back to English.
pub const UI_KEYS: &[&str] = &[
    "tagline", "disclaimer", "skip", "guides", "updated", "official_source", "open_official", "report_error",
    "not_found", "back", "empty", "about", "disclaimer_link", "terms", "privacy", "language", "untranslated",
    "english_badge", "footer_note", "find_title", "find_sub", "find_label", "find_placeholder", "all",
    "count_one", "count_many", "no_match", "how_title", "how_intro", "how1", "how1_value", "how2",
    "how2_value", "how3", "how3_value", "how_final", "how_final_value", "fact1_title", "fact1_text",
    "fact2_title", "fact2_text", "fact3_title", "fact3_text", "faq_title", "faq1_q", "faq1_a", "faq2_q",
    "faq2_a", "faq3_q", "faq3_a", "faq4_q", "faq4_a", "faq5_q", "faq5_a",
];

pub struct Guide {
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub official_name: String,
    pub official_url: String,
    /// ISO date (YYYY-MM-DD) of the last time the content was checked against the official source.
    pub updated: String,
    pub body_html: String,
}

pub struct Page {
    pub slug: String,
    pub title: String,
    pub body_html: String,
}

#[derive(Default)]
pub struct Site {
    guides: HashMap<(Lang, String), Guide>,
    pages: HashMap<String, Page>,
    ui: HashMap<Lang, HashMap<String, String>>,
}

impl Site {
    /// Loads `<root>/guides/<lang>/*.md` and `<root>/pages/*.md`. Any malformed file is an error,
    /// so a typo in front matter fails at startup instead of publishing a broken page.
    pub fn load(root: &Path) -> Result<Site, String> {
        let mut site = Site::default();
        for lang in Lang::all() {
            let file = root.join("ui").join(format!("{}.txt", lang.code()));
            if file.exists() {
                let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
                let map = parse_ui(&text).map_err(|e| format!("ui/{}.txt: {e}", lang.code()))?;
                site.ui.insert(lang, map);
            }
        }
        let missing_en = site.missing_ui_keys(Lang::EN);
        if !missing_en.is_empty() {
            return Err(format!("ui/en.txt is missing keys: {}", missing_en.join(", ")));
        }
        for lang in Lang::all() {
            for (slug, src) in read_markdown_dir(&root.join("guides").join(lang.code()))? {
                let guide = parse_guide(&slug, &src).map_err(|e| format!("guides/{}/{slug}.md: {e}", lang.code()))?;
                site.guides.insert((lang, slug), guide);
            }
        }
        for (slug, src) in read_markdown_dir(&root.join("pages"))? {
            let page = parse_page(&slug, &src).map_err(|e| format!("pages/{slug}.md: {e}"))?;
            site.pages.insert(slug, page);
        }
        Ok(site)
    }

    /// Interface text for `lang`, falling back to English, then to the key itself.
    pub fn t<'a>(&'a self, lang: Lang, key: &'a str) -> &'a str {
        self.ui
            .get(&lang)
            .and_then(|m| m.get(key))
            .or_else(|| self.ui.get(&Lang::EN).and_then(|m| m.get(key)))
            .map(String::as_str)
            .unwrap_or(key)
    }

    pub fn missing_ui_keys(&self, lang: Lang) -> Vec<&'static str> {
        let have = self.ui.get(&lang);
        UI_KEYS.iter().copied().filter(|k| !have.is_some_and(|m| m.contains_key(*k))).collect()
    }

    /// The guide in `lang`, or the English one when it has not been translated (`false` = fallback).
    pub fn guide_or_fallback(&self, lang: Lang, slug: &str) -> Option<(&Guide, bool)> {
        self.guide(lang, slug)
            .map(|g| (g, true))
            .or_else(|| self.guide(Lang::EN, slug).map(|g| (g, false)))
    }

    /// Every guide a reader of `lang` can open: translated ones, plus English ones not yet translated.
    pub fn guides_for_index(&self, lang: Lang) -> Vec<(&Guide, bool)> {
        let mut slugs: Vec<&str> = self.guides.keys().map(|(_, s)| s.as_str()).collect();
        slugs.sort_unstable();
        slugs.dedup();
        let mut list: Vec<(&Guide, bool)> = slugs.into_iter().filter_map(|s| self.guide_or_fallback(lang, s)).collect();
        list.sort_by(|a, b| (&a.0.category, &a.0.title).cmp(&(&b.0.category, &b.0.title)));
        list
    }

    /// One line per language that is incomplete, for the startup log.
    pub fn coverage_report(&self) -> Vec<String> {
        let total = self.guides(Lang::EN).len();
        Lang::all()
            .filter(|l| *l != Lang::EN)
            .filter_map(|l| {
                let ui = self.missing_ui_keys(l).len();
                let guides = self.guides(l).len();
                (ui > 0 || guides < total).then(|| {
                    format!("{}: {} interface strings missing, {guides}/{total} guides translated", l.code(), ui)
                })
            })
            .collect()
    }

    pub fn guide(&self, lang: Lang, slug: &str) -> Option<&Guide> {
        self.guides.get(&(lang, slug.to_string()))
    }

    pub fn page(&self, slug: &str) -> Option<&Page> {
        self.pages.get(slug)
    }

    pub fn guides(&self, lang: Lang) -> Vec<&Guide> {
        let mut list: Vec<&Guide> = self.guides.iter().filter(|((l, _), _)| *l == lang).map(|(_, g)| g).collect();
        list.sort_by(|a, b| (&a.category, &a.title).cmp(&(&b.category, &b.title)));
        list
    }

    pub fn pages(&self) -> Vec<&Page> {
        let mut list: Vec<&Page> = self.pages.values().collect();
        list.sort_by(|a, b| a.slug.cmp(&b.slug));
        list
    }
}

fn read_markdown_dir(dir: &Path) -> Result<Vec<(String, String)>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let slug = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_string();
        if !valid_slug(&slug) {
            return Err(format!("{}: file name must be lowercase letters, digits and '-'", path.display()));
        }
        let src = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        out.push((slug, src));
    }
    Ok(out)
}

/// Parses `key = value` lines. Blank lines and lines starting with '#' are ignored.
fn parse_ui(text: &str) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or(format!("line {}: expected 'key = value'", n + 1))?;
        let key = key.trim();
        if !UI_KEYS.contains(&key) {
            return Err(format!("line {}: unknown key '{key}'", n + 1));
        }
        map.insert(key.to_string(), value.trim().to_string());
    }
    Ok(map)
}

pub fn valid_slug(slug: &str) -> bool {
    !slug.is_empty() && slug.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn parse_guide(slug: &str, src: &str) -> Result<Guide, String> {
    let (mut meta, body) = split_front_matter(src)?;
    let mut take = |key: &str| meta.remove(key).filter(|v| !v.is_empty()).ok_or(format!("missing '{key}' in front matter"));
    let guide = Guide {
        slug: slug.to_string(),
        title: take("title")?,
        summary: take("summary")?,
        category: take("category")?,
        official_name: take("official_name")?,
        official_url: take("official_url")?,
        updated: take("updated")?,
        body_html: render_markdown(body),
    };
    if !guide.official_url.starts_with("https://") {
        return Err("official_url must start with https://".into());
    }
    if !valid_date(&guide.updated) {
        return Err("updated must look like YYYY-MM-DD".into());
    }
    Ok(guide)
}

fn parse_page(slug: &str, src: &str) -> Result<Page, String> {
    let (meta, body) = split_front_matter(src)?;
    let title = meta.get("title").filter(|t| !t.is_empty()).ok_or("missing 'title' in front matter")?.clone();
    Ok(Page { slug: slug.to_string(), title, body_html: render_markdown(body) })
}

fn valid_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10 && b.iter().enumerate().all(|(i, c)| if i == 4 || i == 7 { *c == b'-' } else { c.is_ascii_digit() })
}

/// Splits `---\nkey: value\n---\nbody` into the key/value map and the body.
fn split_front_matter(src: &str) -> Result<(HashMap<String, String>, &str), String> {
    let src = src.trim_start_matches('\u{feff}');
    let rest = src
        .strip_prefix("---\n")
        .or_else(|| src.strip_prefix("---\r\n"))
        .ok_or("file must start with a '---' front matter block")?;
    let mut meta = HashMap::new();
    let mut consumed = 0;
    for line in rest.split_inclusive('\n') {
        consumed += line.len();
        let line = line.trim_end_matches(['\n', '\r']);
        if line == "---" {
            return Ok((meta, &rest[consumed..]));
        }
        if line.trim().is_empty() {
            continue;
        }
        let (key, value) = line.split_once(':').ok_or(format!("bad front matter line: '{line}'"))?;
        meta.insert(key.trim().to_string(), value.trim().to_string());
    }
    Err("front matter block is not closed with '---'".into())
}

/// Markdown to HTML. Raw HTML in the source is shown as text, and links may only be
/// http(s), mailto or relative, so content files cannot inject markup or scripts.
fn render_markdown(md: &str) -> String {
    let events = Parser::new_ext(md, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH).map(|event| match event {
        Event::Html(t) | Event::InlineHtml(t) => Event::Text(t),
        Event::Start(Tag::Link { link_type, dest_url, title, id }) => {
            let lower = dest_url.to_ascii_lowercase();
            let safe = lower.starts_with("https://")
                || lower.starts_with("http://")
                || lower.starts_with("mailto:")
                || lower.starts_with('/')
                || lower.starts_with('#');
            let dest_url = if safe { dest_url } else { CowStr::from("#") };
            Event::Start(Tag::Link { link_type, dest_url, title, id })
        }
        other => other,
    });
    let mut out = String::new();
    html::push_html(&mut out, events);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const GUIDE: &str = "---\ntitle: T\nsummary: S\ncategory: C\nofficial_name: O\nofficial_url: https://example.gov.in\nupdated: 2026-01-31\n---\n# Hi\n";

    #[test]
    fn parses_a_valid_guide() {
        let g = parse_guide("x", GUIDE).unwrap();
        assert_eq!(g.title, "T");
        assert!(g.body_html.contains("<h1>Hi</h1>"));
    }

    #[test]
    fn rejects_missing_fields_and_bad_values() {
        assert!(parse_guide("x", "no front matter").is_err());
        assert!(parse_guide("x", &GUIDE.replace("title: T\n", "")).is_err());
        assert!(parse_guide("x", &GUIDE.replace("https://", "http://")).is_err());
        assert!(parse_guide("x", &GUIDE.replace("2026-01-31", "yesterday")).is_err());
    }

    #[test]
    fn markdown_cannot_inject_html_or_script_links() {
        let html = render_markdown("<script>alert(1)</script>\n\n[x](javascript:alert(1))");
        assert!(!html.contains("<script>"));
        assert!(!html.contains("javascript:"));
    }

    #[test]
    fn accept_language_picks_best_supported_language() {
        assert_eq!(Lang::from_accept_language("hi-IN,hi;q=0.9,en;q=0.8"), Lang::from_code("hi"));
        assert_eq!(Lang::from_accept_language("fr,ta;q=0.5,en;q=0.2"), Lang::from_code("ta"));
        assert_eq!(Lang::from_accept_language("en;q=0.2, ur;q=0.9"), Lang::from_code("ur"));
        assert_eq!(Lang::from_accept_language("fr,de"), None);
        assert_eq!(Lang::from_accept_language("hi;q=0"), None);
    }

    #[test]
    fn ui_files_reject_unknown_keys() {
        assert!(parse_ui("tagline = ok\n# note\n\n").is_ok());
        assert!(parse_ui("taglnie = typo").is_err());
        assert!(parse_ui("no equals sign").is_err());
    }

    #[test]
    fn slugs_are_restricted() {
        assert!(valid_slug("aadhaar-update"));
        assert!(!valid_slug("../etc/passwd"));
        assert!(!valid_slug(""));
        assert!(!valid_slug("Upper"));
    }
}
