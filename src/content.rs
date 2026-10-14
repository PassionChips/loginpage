use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Lang {
    En,
    Ta,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::En, Lang::Ta];

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ta => "ta",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|l| l.code() == code)
    }

    pub fn native_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Ta => "தமிழ்",
        }
    }
}

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
}

impl Site {
    /// Loads `<root>/guides/<lang>/*.md` and `<root>/pages/*.md`. Any malformed file is an error,
    /// so a typo in front matter fails at startup instead of publishing a broken page.
    pub fn load(root: &Path) -> Result<Site, String> {
        let mut site = Site::default();
        for lang in Lang::ALL {
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
    fn slugs_are_restricted() {
        assert!(valid_slug("aadhaar-update"));
        assert!(!valid_slug("../etc/passwd"));
        assert!(!valid_slug(""));
        assert!(!valid_slug("Upper"));
    }
}
